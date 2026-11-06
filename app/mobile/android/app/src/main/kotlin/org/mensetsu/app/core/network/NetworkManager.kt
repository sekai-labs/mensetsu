package org.mensetsu.app.core.network

import android.content.ContentResolver
import android.database.Cursor
import android.net.Uri
import android.provider.OpenableColumns
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import okhttp3.Interceptor
import okhttp3.CookieJar
import okhttp3.MediaType
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody
import okhttp3.Response
import okhttp3.logging.HttpLoggingInterceptor
import okio.BufferedSink
import okio.source
import org.mensetsu.app.core.storage.TokenStorage
import org.mensetsu.contracts.apis.AuthApi
import org.mensetsu.contracts.apis.ResumeUploadsApi
import org.mensetsu.contracts.apis.SettingsApi
import org.mensetsu.contracts.apis.WorkflowsApi
import org.mensetsu.contracts.apis.WorkspacesApi
import org.mensetsu.contracts.apis.WorkspacesLearningHandlersApi
import org.mensetsu.contracts.apis.WorkspacesResearchHandlersApi
import org.mensetsu.contracts.models.RefreshTokenRequestDto
import org.mensetsu.contracts.models.CreateResumeUploadRequestDto
import org.mensetsu.contracts.models.CreateWorkspaceRequestDto
import org.mensetsu.contracts.models.WorkspaceResponseDto
import java.io.IOException
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

data class SelectedPdf(val uri: Uri, val filename: String, val sizeBytes: Long)

class NetworkManager(private val tokenStorage: TokenStorage) {

    private val memoryAccessToken = AtomicReference<String?>(null)
    private val refreshMutex = Mutex()

    private val sessionLock = Any()
    private var sessionEpoch: Long = 0L

    fun getAccessToken(): String? = memoryAccessToken.get()

    fun setAccessToken(token: String?) {
        memoryAccessToken.set(token)
    }

    fun getRefreshToken(): String? = tokenStorage.getRefreshToken()

    fun saveRefreshToken(token: String?) {
        tokenStorage.saveRefreshToken(token)
    }

    fun clearSession() {
        synchronized(sessionLock) {
            sessionEpoch++
            memoryAccessToken.set(null)
            tokenStorage.clearRefreshToken()
        }
    }

    fun isEndpointConfigured(): Boolean {
        val custom = tokenStorage.getCustomBaseUrl()
        if (!custom.isNullOrBlank()) {
            return EndpointValidator.validate(custom) is EndpointValidator.ValidationResult.Valid
        }
        return false
    }

    fun getBaseUrl(): String {
        val custom = tokenStorage.getCustomBaseUrl()
        if (!custom.isNullOrBlank()) {
            val res = EndpointValidator.validate(custom)
            if (res is EndpointValidator.ValidationResult.Valid) return res.normalizedUrl
        }
        return ""
    }

    fun setBaseUrl(url: String?) {
        if (url.isNullOrBlank()) {
            synchronized(sessionLock) {
                sessionEpoch++
                memoryAccessToken.set(null)
                tokenStorage.clearRefreshToken()
                tokenStorage.clearCustomBaseUrl()
            }
            return
        }

        when (val res = EndpointValidator.validate(url)) {
            is EndpointValidator.ValidationResult.Valid -> {
                synchronized(sessionLock) {
                    sessionEpoch++
                    memoryAccessToken.set(null)
                    tokenStorage.clearRefreshToken()
                    tokenStorage.setCustomBaseUrl(res.normalizedUrl)
                }
            }
            is EndpointValidator.ValidationResult.Invalid -> {
                throw IllegalArgumentException("Invalid server endpoint: ${res.reason}")
            }
        }
    }

    private val authInterceptor = Interceptor { chain ->
        val originalRequest = chain.request()

        val path = originalRequest.url.encodedPath
        val isAuthEndpoint = path.contains("/auth/login") ||
                path.contains("/auth/register") ||
                path.contains("/auth/refresh") ||
                path.contains("/auth/login-2fa")

        val requestEpoch: Long
        val currentAccess: String?
        val requestBaseUrl: String
        synchronized(sessionLock) {
            requestEpoch = sessionEpoch
            currentAccess = memoryAccessToken.get()
            requestBaseUrl = getBaseUrl()
        }

        val requestWithAuth = if (!isAuthEndpoint && !currentAccess.isNullOrBlank()) {
            originalRequest.newBuilder()
                .header("Authorization", "Bearer $currentAccess")
                .build()
        } else {
            originalRequest
        }

        val response = chain.proceed(requestWithAuth)

        if (response.code == 401 && !isAuthEndpoint) {
            synchronized(sessionLock) {
                if (sessionEpoch != requestEpoch || requestBaseUrl.isBlank()) {
                    return@Interceptor response
                }
            }

            val refreshBaseUrl = requestBaseUrl

            val newAccessToken = kotlinx.coroutines.runBlocking {
                refreshMutex.withLock {
                    synchronized(sessionLock) {
                        if (sessionEpoch != requestEpoch) {
                            return@withLock null
                        }
                    }

                    val candidate = memoryAccessToken.get()
                    if (candidate != null && candidate != currentAccess) {
                        return@withLock candidate
                    }

                    val refreshToken = tokenStorage.getRefreshToken()
                    if (refreshToken.isNullOrBlank()) {
                        clearSession()
                        return@withLock null
                    }

                    try {
                        val authApi = AuthApi(basePath = refreshBaseUrl, client = rawOkHttpClient)
                        val tokenResponse = authApi.refresh(
                            RefreshTokenRequestDto(refreshToken = refreshToken)
                        )
                        val newAccess = tokenResponse.accessToken
                        val newRefresh = tokenResponse.refreshToken

                        synchronized(sessionLock) {
                            if (sessionEpoch != requestEpoch) {
                                return@withLock null
                            }
                            saveRefreshToken(newRefresh)
                            setAccessToken(newAccess)
                        }
                        newAccess
                    } catch (e: Exception) {
                        synchronized(sessionLock) {
                            if (sessionEpoch == requestEpoch) {
                                clearSession()
                            }
                        }
                        null
                    }
                }
            }
            val shouldRetry: Boolean
            synchronized(sessionLock) {
                shouldRetry = (!newAccessToken.isNullOrBlank() && sessionEpoch == requestEpoch)
            }

            if (shouldRetry) {
                response.close()
                val retryRequest = originalRequest.newBuilder()
                    .header("Authorization", "Bearer $newAccessToken")
                    .build()
                return@Interceptor chain.proceed(retryRequest)
            }
        }

        response
    }

    val rawOkHttpClient: OkHttpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .writeTimeout(30, TimeUnit.SECONDS)
        .addInterceptor(HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BASIC
        })
        .build()

    val okHttpClient: OkHttpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .writeTimeout(30, TimeUnit.SECONDS)
        .addInterceptor(authInterceptor)
        .addInterceptor(HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BASIC
        })
        .build()

    private val signedUploadClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(60, TimeUnit.SECONDS)
        .writeTimeout(60, TimeUnit.SECONDS)
        .cookieJar(CookieJar.NO_COOKIES)
        .followRedirects(false)
        .followSslRedirects(false)
        .build()

    fun inspectPdf(contentResolver: ContentResolver, uri: Uri): SelectedPdf {
        var filename: String? = null
        var sizeBytes: Long? = null
        contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)
            ?.use { cursor: Cursor ->
                if (cursor.moveToFirst()) {
                    val nameColumn = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    val sizeColumn = cursor.getColumnIndex(OpenableColumns.SIZE)
                    if (nameColumn >= 0 && !cursor.isNull(nameColumn)) {
                        filename = cursor.getString(nameColumn)
                    }
                    if (sizeColumn >= 0 && !cursor.isNull(sizeColumn)) {
                        sizeBytes = cursor.getLong(sizeColumn)
                    }
                }
            }
        val pdfName = filename ?: uri.lastPathSegment.orEmpty()
        require(pdfName.length > 4 && pdfName.endsWith(".pdf", ignoreCase = true)) {
            "Choose a PDF résumé."
        }
        if (sizeBytes == null || sizeBytes!! < 0) {
            contentResolver.openInputStream(uri)?.use { input ->
                val buffer = ByteArray(8192)
                var measured = 0L
                while (true) {
                    val read = input.read(buffer)
                    if (read < 0) break
                    measured += read
                    require(measured <= MAX_PDF_BYTES) { "Résumé PDF must be at most 5 MiB." }
                }
                sizeBytes = measured
            } ?: throw IOException("Cannot read the selected PDF.")
        }
        val size = sizeBytes!!
        require(size in 1..MAX_PDF_BYTES) { "Résumé PDF must be between 1 byte and 5 MiB." }
        return SelectedPdf(uri, pdfName, size)
    }

    fun createWorkspaceWithPdf(
        contentResolver: ContentResolver,
        selected: SelectedPdf,
        company: String,
        role: String,
        jobDescription: String
    ): WorkspaceResponseDto {
        val upload = getResumeUploadsApi().createResumeUpload(
            CreateResumeUploadRequestDto(
                contentType = "application/pdf",
                filename = selected.filename,
                sizeBytes = selected.sizeBytes
            )
        )
        val pdfBody = object : RequestBody() {
            override fun contentType(): MediaType = "application/pdf".toMediaType()
            override fun contentLength(): Long = selected.sizeBytes
            override fun writeTo(sink: BufferedSink) {
                val written = contentResolver.openInputStream(selected.uri)?.use { input ->
                    input.source().use { source -> sink.writeAll(source) }
                } ?: throw IOException("Cannot read the selected PDF.")
                if (written != selected.sizeBytes) {
                    throw IOException("Selected PDF changed size during upload.")
                }
            }
        }
        val put = Request.Builder()
            .url(upload.uploadUrl)
            .put(pdfBody)
            .build()
        signedUploadClient.newCall(put).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("PDF storage upload failed: HTTP ${response.code}")
            }
        }
        val completed = getResumeUploadsApi().completeResumeUpload(upload.uploadId)
        check(completed.status == "completed") { "PDF storage confirmation did not complete." }
        return getWorkspacesApi().createWorkspace(
            CreateWorkspaceRequestDto(
                companyName = company,
                roleTitle = role,
                resumeUploadId = completed.uploadId,
                jobDescription = jobDescription.ifBlank { null }
            )
        )
    }

    private companion object {
        const val MAX_PDF_BYTES: Long = 5L * 1024 * 1024
    }


    fun getAuthApi(): AuthApi = AuthApi(basePath = getBaseUrl(), client = okHttpClient)
    fun getWorkspacesApi(): WorkspacesApi = WorkspacesApi(basePath = getBaseUrl(), client = okHttpClient)
    fun getLearningHandlersApi(): WorkspacesLearningHandlersApi =
        WorkspacesLearningHandlersApi(basePath = getBaseUrl(), client = okHttpClient)
    fun getResumeUploadsApi(): ResumeUploadsApi =
        ResumeUploadsApi(basePath = getBaseUrl(), client = okHttpClient)
    fun getResearchHandlersApi(): WorkspacesResearchHandlersApi =
        WorkspacesResearchHandlersApi(basePath = getBaseUrl(), client = okHttpClient)
    fun getSettingsApi(): SettingsApi = SettingsApi(basePath = getBaseUrl(), client = okHttpClient)
    fun getWorkflowsApi(): WorkflowsApi = WorkflowsApi(basePath = getBaseUrl(), client = okHttpClient)
}

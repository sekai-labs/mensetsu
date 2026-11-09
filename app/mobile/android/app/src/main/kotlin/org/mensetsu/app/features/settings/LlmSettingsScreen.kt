package org.mensetsu.app.features.settings

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.material3.TabRowDefaults.tabIndicatorOffset
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.mensetsu.app.core.network.EndpointValidator
import org.mensetsu.app.core.network.NetworkManager
import org.mensetsu.app.core.tags.AppTestTags
import org.mensetsu.app.core.theme.*
import org.mensetsu.contracts.models.LlmConfigItemDto
import org.mensetsu.contracts.models.TokenUsageCursorDto
import org.mensetsu.contracts.models.TokenUsageResponseDto
import org.mensetsu.contracts.models.TokenUsageRowDto
import org.mensetsu.contracts.models.UpsertLlmConfigRequestDto
import java.text.NumberFormat

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LlmSettingsScreen(
    networkManager: NetworkManager,
    onBack: (() -> Unit)? = null,
    onEndpointChanged: (() -> Unit)? = null
) {
    var selectedTab by remember { mutableIntStateOf(0) }

    var settings by remember { mutableStateOf<List<LlmConfigItemDto>>(emptyList()) }
    var isLoadingSettings by remember { mutableStateOf(true) }
    var settingsErrorMessage by remember { mutableStateOf<String?>(null) }
    var editingConfig by remember { mutableStateOf<LlmConfigItemDto?>(null) }

    var endpointInput by remember { mutableStateOf(networkManager.getBaseUrl()) }
    var endpointError by remember { mutableStateOf<String?>(null) }
    var endpointSuccessMessage by remember { mutableStateOf<String?>(null) }

    var tokenUsageResponse by remember { mutableStateOf<TokenUsageResponseDto?>(null) }
    var olderRows by remember { mutableStateOf<List<TokenUsageRowDto>>(emptyList()) }
    var isTokenLoading by remember { mutableStateOf(false) }
    var tokenErrorMessage by remember { mutableStateOf<String?>(null) }
    var isLoadingOlder by remember { mutableStateOf(false) }
    var olderErrorMessage by remember { mutableStateOf<String?>(null) }
    var nextCursor by remember { mutableStateOf<TokenUsageCursorDto?>(null) }

    val coroutineScope = rememberCoroutineScope()

    fun loadSettings() {
        isLoadingSettings = true
        settingsErrorMessage = null
        coroutineScope.launch {
            try {
                val list = withContext(Dispatchers.IO) {
                    networkManager.getSettingsApi().getLlmSettings()
                }
                settings = list
                isLoadingSettings = false
            } catch (e: Exception) {
                isLoadingSettings = false
                settingsErrorMessage = e.message ?: "Failed to load LLM settings."
            }
        }
    }

    fun loadTokenUsage() {
        if (isTokenLoading || isLoadingOlder) return
        isTokenLoading = true
        tokenErrorMessage = null
        olderRows = emptyList()
        olderErrorMessage = null
        coroutineScope.launch {
            try {
                val res = withContext(Dispatchers.IO) {
                    networkManager.getTokenUsageApi().userTokenUsage()
                }
                tokenUsageResponse = res
                nextCursor = res.nextCursor
            } catch (e: Exception) {
                tokenErrorMessage = e.message ?: "Failed to load token usage."
            } finally {
                isTokenLoading = false
            }
        }
    }

    fun loadOlderTokens() {
        if (isTokenLoading || isLoadingOlder) return
        val cursor = nextCursor ?: return
        isLoadingOlder = true
        olderErrorMessage = null
        coroutineScope.launch {
            try {
                val res = withContext(Dispatchers.IO) {
                    networkManager.getTokenUsageApi().userTokenUsage(
                        beforeTimestamp = cursor.beforeTimestamp,
                        beforeEventId = cursor.beforeEventId
                    )
                }
                olderRows = olderRows + res.items
                nextCursor = res.nextCursor
                tokenUsageResponse = tokenUsageResponse?.copy(totals = res.totals)
            } catch (e: Exception) {
                olderErrorMessage = e.message ?: "Failed to load older requests."
            } finally {
                isLoadingOlder = false
            }
        }
    }

    LaunchedEffect(Unit) {
        loadSettings()
    }

    LaunchedEffect(selectedTab) {
        if (selectedTab == 1 && tokenUsageResponse == null && !isTokenLoading) {
            loadTokenUsage()
        }
    }

    Scaffold(
        modifier = Modifier.testTag(AppTestTags.SETTINGS_SCREEN),
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                navigationIcon = {
                    if (onBack != null) {
                        IconButton(onClick = onBack) {
                            Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                        }
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            TabRow(
                selectedTabIndex = selectedTab,
                containerColor = MaterialTheme.colorScheme.surface,
                contentColor = PrimaryIndigo,
                indicator = { tabPositions ->
                    TabRowDefaults.SecondaryIndicator(
                        modifier = Modifier.tabIndicatorOffset(tabPositions[selectedTab]),
                        color = PrimaryIndigo
                    )
                }
            ) {
                Tab(
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 },
                    text = {
                        Text(
                            text = "Providers",
                            fontWeight = if (selectedTab == 0) FontWeight.SemiBold else FontWeight.Normal
                        )
                    }
                )
                Tab(
                    selected = selectedTab == 1,
                    onClick = { selectedTab = 1 },
                    text = {
                        Text(
                            text = "Token Usage",
                            fontWeight = if (selectedTab == 1) FontWeight.SemiBold else FontWeight.Normal
                        )
                    }
                )
            }

            if (selectedTab == 0) {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    item {
                        Text(
                            text = "Server Configuration",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Configured self-hosted Rust API endpoint. Changing this clears credentials and resets your session.",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedCard(
                            modifier = Modifier.fillMaxWidth(),
                            border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline)
                        ) {
                            Column(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(16.dp)
                            ) {
                                if (endpointError != null) {
                                    Text(
                                        text = endpointError ?: "",
                                        color = MaterialTheme.colorScheme.error,
                                        style = MaterialTheme.typography.bodySmall,
                                        modifier = Modifier.testTag(AppTestTags.SERVER_URL_ERROR_MESSAGE)
                                    )
                                    Spacer(modifier = Modifier.height(8.dp))
                                }
                                if (endpointSuccessMessage != null) {
                                    Text(
                                        text = endpointSuccessMessage ?: "",
                                        color = EmeraldGreen,
                                        style = MaterialTheme.typography.bodySmall
                                    )
                                    Spacer(modifier = Modifier.height(8.dp))
                                }
                                OutlinedTextField(
                                    value = endpointInput,
                                    onValueChange = {
                                        endpointInput = it
                                        endpointError = null
                                        endpointSuccessMessage = null
                                    },
                                    label = { Text("Server Base URL") },
                                    singleLine = true,
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .testTag(AppTestTags.SETTINGS_INPUT_SERVER_URL)
                                )
                                Spacer(modifier = Modifier.height(12.dp))
                                Button(
                                    onClick = {
                                        endpointError = null
                                        endpointSuccessMessage = null
                                        when (val res = EndpointValidator.validate(endpointInput)) {
                                            is EndpointValidator.ValidationResult.Valid -> {
                                                val newUrl = res.normalizedUrl
                                                if (networkManager.getBaseUrl() == newUrl) {
                                                    endpointSuccessMessage = "Server URL unchanged."
                                                } else {
                                                    networkManager.setBaseUrl(newUrl)
                                                    endpointInput = newUrl
                                                    onEndpointChanged?.invoke()
                                                }
                                            }
                                            is EndpointValidator.ValidationResult.Invalid -> {
                                                endpointError = res.reason
                                            }
                                        }
                                    },
                                    modifier = Modifier.testTag(AppTestTags.SETTINGS_BUTTON_SAVE_SERVER_URL)
                                ) {
                                    Text("Update Server URL")
                                }
                            }
                        }
                    }

                    item {
                        HorizontalDivider(color = MaterialTheme.colorScheme.outline)
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "LLM Provider Configuration",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Configure underlying LLM provider keys for research synthesis and interview questions.",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    when {
                        isLoadingSettings -> {
                            item {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 32.dp),
                                    contentAlignment = Alignment.Center
                                ) {
                                    CircularProgressIndicator(color = PrimaryIndigo)
                                }
                            }
                        }
                        settingsErrorMessage != null -> {
                            item {
                                OutlinedCard(
                                    modifier = Modifier.fillMaxWidth(),
                                    border = BorderStroke(1.dp, RoseRed.copy(alpha = 0.5f))
                                ) {
                                    Column(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(16.dp),
                                        horizontalAlignment = Alignment.CenterHorizontally
                                    ) {
                                        Text(
                                            text = settingsErrorMessage ?: "Error loading settings",
                                            color = MaterialTheme.colorScheme.error,
                                            style = MaterialTheme.typography.bodySmall
                                        )
                                        Spacer(modifier = Modifier.height(12.dp))
                                        Button(onClick = { loadSettings() }) {
                                            Text("Retry")
                                        }
                                    }
                                }
                            }
                        }
                        settings.isEmpty() -> {
                            item {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 24.dp),
                                    contentAlignment = Alignment.Center
                                ) {
                                    Text(
                                        text = "No LLM providers configured yet.",
                                        style = MaterialTheme.typography.bodyMedium,
                                        color = Slate500
                                    )
                                }
                            }
                        }
                        else -> {
                            items(settings, key = { it.provider }) { config ->
                                OutlinedCard(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .testTag("${AppTestTags.SETTINGS_PROVIDER_PREFIX}${config.provider}"),
                                    border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline)
                                ) {
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(14.dp),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Column(modifier = Modifier.weight(1f)) {
                                            Text(
                                                text = config.provider.uppercase(),
                                                style = MaterialTheme.typography.titleSmall,
                                                fontWeight = FontWeight.Bold
                                            )
                                            Spacer(modifier = Modifier.height(4.dp))
                                            if (config.hasKey) {
                                                Row(verticalAlignment = Alignment.CenterVertically) {
                                                    Icon(
                                                        Icons.Default.Check,
                                                        contentDescription = "Active",
                                                        tint = EmeraldGreen,
                                                        modifier = Modifier.size(15.dp)
                                                    )
                                                    Spacer(modifier = Modifier.width(4.dp))
                                                    Text(
                                                        text = config.maskedKey ?: "Key Configured",
                                                        fontFamily = FontFamily.Monospace,
                                                        fontSize = 12.sp,
                                                        color = EmeraldGreen
                                                    )
                                                }
                                            } else {
                                                Text(
                                                    text = "Not Configured",
                                                    fontSize = 12.sp,
                                                    color = Slate400
                                                )
                                            }
                                            if (!config.modelChat.isNullOrBlank()) {
                                                Spacer(modifier = Modifier.height(2.dp))
                                                Text(
                                                    text = "Chat Model: ${config.modelChat}",
                                                    fontFamily = FontFamily.Monospace,
                                                    fontSize = 11.sp,
                                                    color = Slate500
                                                )
                                            }
                                        }

                                        Button(
                                            onClick = { editingConfig = config },
                                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp)
                                        ) {
                                            Icon(
                                                Icons.Default.Edit,
                                                contentDescription = "Edit Key",
                                                modifier = Modifier.size(14.dp)
                                            )
                                            Spacer(modifier = Modifier.width(4.dp))
                                            Text(if (config.hasKey) "Update" else "Set Key", fontSize = 12.sp)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    item {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = "Token Usage Analytics",
                                    style = MaterialTheme.typography.titleMedium,
                                    fontWeight = FontWeight.Bold
                                )
                                Spacer(modifier = Modifier.height(2.dp))
                                Text(
                                    text = "Per-model-request totals from recorded provider responses.",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                            Spacer(modifier = Modifier.width(8.dp))
                            OutlinedButton(
                                onClick = { loadTokenUsage() },
                                enabled = !isTokenLoading && !isLoadingOlder,
                                contentPadding = PaddingValues(horizontal = 10.dp, vertical = 6.dp)
                            ) {
                                if (isTokenLoading) {
                                    CircularProgressIndicator(
                                        modifier = Modifier.size(14.dp),
                                        strokeWidth = 2.dp,
                                        color = PrimaryIndigo
                                    )
                                } else {
                                    Icon(
                                        Icons.Default.Refresh,
                                        contentDescription = "Refresh",
                                        modifier = Modifier.size(14.dp)
                                    )
                                }
                                Spacer(modifier = Modifier.width(4.dp))
                                Text("Refresh", fontSize = 12.sp)
                            }
                        }
                    }

                    item {
                        val totals = tokenUsageResponse?.totals
                        Column(
                            modifier = Modifier.fillMaxWidth(),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                KpiCard(
                                    modifier = Modifier.weight(1f),
                                    title = "REQUESTS",
                                    value = formatNumber(totals?.requestCount ?: 0L),
                                    isAccent = false
                                )
                                KpiCard(
                                    modifier = Modifier.weight(1f),
                                    title = "INPUT TOKENS",
                                    value = formatNumber(totals?.inputTokens ?: 0L),
                                    isAccent = false
                                )
                            }
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                KpiCard(
                                    modifier = Modifier.weight(1f),
                                    title = "OUTPUT TOKENS",
                                    value = formatNumber(totals?.outputTokens ?: 0L),
                                    isAccent = false
                                )
                                KpiCard(
                                    modifier = Modifier.weight(1f),
                                    title = "ALL TOKENS",
                                    value = formatNumber(totals?.totalTokens ?: 0L),
                                    isAccent = true
                                )
                            }
                        }
                    }

                    item {
                        HorizontalDivider(color = MaterialTheme.colorScheme.outline)
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Request History",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                    }

                    when {
                        isTokenLoading && tokenUsageResponse == null -> {
                            item {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 32.dp),
                                    contentAlignment = Alignment.Center
                                ) {
                                    CircularProgressIndicator(color = PrimaryIndigo)
                                }
                            }
                        }
                        tokenErrorMessage != null && tokenUsageResponse == null -> {
                            item {
                                OutlinedCard(
                                    modifier = Modifier.fillMaxWidth(),
                                    border = BorderStroke(1.dp, RoseRed.copy(alpha = 0.5f))
                                ) {
                                    Column(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(16.dp),
                                        horizontalAlignment = Alignment.CenterHorizontally
                                    ) {
                                        Text(
                                            text = tokenErrorMessage ?: "Failed to load token usage",
                                            color = MaterialTheme.colorScheme.error,
                                            style = MaterialTheme.typography.bodySmall
                                        )
                                        Spacer(modifier = Modifier.height(12.dp))
                                        Button(onClick = { loadTokenUsage() }) {
                                            Text("Retry")
                                        }
                                    }
                                }
                            }
                        }
                        else -> {
                            if (tokenErrorMessage != null) {
                                item {
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(vertical = 8.dp),
                                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Text(
                                            text = tokenErrorMessage ?: "",
                                            color = MaterialTheme.colorScheme.error,
                                            modifier = Modifier.weight(1f)
                                        )
                                        TextButton(onClick = { loadTokenUsage() }) {
                                            Text("Retry")
                                        }
                                    }
                                }
                            }
                            val allRows = (tokenUsageResponse?.items.orEmpty()) + olderRows
                            if (allRows.isEmpty()) {
                                item {
                                    Box(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(vertical = 24.dp),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Text(
                                            text = "No model request usage has been recorded yet.",
                                            style = MaterialTheme.typography.bodyMedium,
                                            color = Slate500
                                        )
                                    }
                                }
                            } else {
                                item {
                                    Surface(
                                        modifier = Modifier.fillMaxWidth(),
                                        color = MaterialTheme.colorScheme.surfaceVariant,
                                        shape = RoundedCornerShape(topStart = 6.dp, topEnd = 6.dp),
                                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline)
                                    ) {
                                        Row(
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .padding(horizontal = 12.dp, vertical = 6.dp),
                                            horizontalArrangement = Arrangement.SpaceBetween
                                        ) {
                                            Text(
                                                text = "MODEL / AGENT",
                                                fontSize = 10.sp,
                                                fontWeight = FontWeight.Bold,
                                                color = Slate500
                                            )
                                            Text(
                                                text = "TOKENS (TOTAL)",
                                                fontSize = 10.sp,
                                                fontWeight = FontWeight.Bold,
                                                color = Slate500
                                            )
                                        }
                                    }
                                }

                                items(allRows, key = { it.id }) { row ->
                                    Surface(
                                        modifier = Modifier.fillMaxWidth(),
                                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline)
                                    ) {
                                        TokenUsageHistoryRow(item = row)
                                    }
                                }

                                item {
                                    Column(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(vertical = 8.dp),
                                        verticalArrangement = Arrangement.spacedBy(8.dp)
                                    ) {
                                        Text(
                                            text = "${allRows.size} requests loaded; aggregate covers ${formatNumber(tokenUsageResponse?.totals?.requestCount ?: 0L)}.",
                                            fontSize = 11.sp,
                                            color = Slate500
                                        )

                                        if (olderErrorMessage != null) {
                                            Text(
                                                text = olderErrorMessage ?: "",
                                                color = MaterialTheme.colorScheme.error,
                                                fontSize = 12.sp
                                            )
                                        }

                                        if (nextCursor != null) {
                                            OutlinedButton(
                                                onClick = { loadOlderTokens() },
                                                enabled = !isLoadingOlder && !isTokenLoading,
                                                modifier = Modifier.fillMaxWidth()
                                            ) {
                                                if (isLoadingOlder) {
                                                    CircularProgressIndicator(
                                                        modifier = Modifier.size(16.dp),
                                                        strokeWidth = 2.dp,
                                                        color = PrimaryIndigo
                                                    )
                                                    Spacer(modifier = Modifier.width(6.dp))
                                                    Text("Loading older requests...")
                                                } else {
                                                    Text("Load older requests")
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if (editingConfig != null) {
        val config = editingConfig!!
        var apiKey by remember { mutableStateOf("") }
        var isSaving by remember { mutableStateOf(false) }
        var saveError by remember { mutableStateOf<String?>(null) }

        AlertDialog(
            onDismissRequest = { if (!isSaving) editingConfig = null },
            title = { Text("Configure ${config.provider.uppercase()} Key") },
            text = {
                Column {
                    Text("Provide custom API key for ${config.provider}:")
                    Spacer(modifier = Modifier.height(12.dp))
                    if (saveError != null) {
                        Text(
                            text = saveError ?: "",
                            color = MaterialTheme.colorScheme.error,
                            style = MaterialTheme.typography.bodySmall
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                    }
                    OutlinedTextField(
                        value = apiKey,
                        onValueChange = { apiKey = it },
                        label = { Text("API Key") },
                        visualTransformation = PasswordVisualTransformation(),
                        singleLine = true,
                        modifier = Modifier
                            .fillMaxWidth()
                            .testTag(AppTestTags.SETTINGS_KEY_INPUT)
                    )
                }
            },
            confirmButton = {
                Button(
                    onClick = {
                        isSaving = true
                        saveError = null
                        coroutineScope.launch {
                            try {
                                val req = UpsertLlmConfigRequestDto(
                                    provider = config.provider,
                                    apiKey = apiKey.trim(),
                                    isShared = config.isShared
                                )
                                withContext(Dispatchers.IO) {
                                    networkManager.getSettingsApi().upsertLlmSetting(req)
                                }
                                isSaving = false
                                editingConfig = null
                                loadSettings()
                            } catch (e: Exception) {
                                isSaving = false
                                saveError = e.message ?: "Failed to save API key."
                            }
                        }
                    },
                    enabled = !isSaving && apiKey.isNotBlank(),
                    modifier = Modifier.testTag(AppTestTags.SETTINGS_SAVE_BUTTON)
                ) {
                    if (isSaving) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(18.dp),
                            color = MaterialTheme.colorScheme.onPrimary
                        )
                    } else {
                        Text("Save")
                    }
                }
            },
            dismissButton = {
                TextButton(
                    onClick = { editingConfig = null },
                    enabled = !isSaving
                ) {
                    Text("Cancel")
                }
            }
        )
    }
}

@Composable
private fun KpiCard(
    modifier: Modifier = Modifier,
    title: String,
    value: String,
    isAccent: Boolean = false
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(6.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
        color = MaterialTheme.colorScheme.surface
    ) {
        Column(
            modifier = Modifier.padding(10.dp)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.labelSmall,
                color = Slate500,
                fontWeight = FontWeight.SemiBold,
                fontSize = 10.sp
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = value,
                fontFamily = FontFamily.Monospace,
                fontWeight = FontWeight.SemiBold,
                fontSize = 16.sp,
                color = if (isAccent) PrimaryIndigo else MaterialTheme.colorScheme.onSurface
            )
        }
    }
}

@Composable
private fun TokenUsageHistoryRow(
    item: TokenUsageRowDto
) {
    val numberFormat = remember { NumberFormat.getIntegerInstance() }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp, horizontal = 12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.weight(1f, fill = false)
            ) {
                Text(
                    text = item.modelName,
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(modifier = Modifier.width(6.dp))
                Surface(
                    shape = RoundedCornerShape(3.dp),
                    color = MaterialTheme.colorScheme.surfaceVariant
                ) {
                    Text(
                        text = item.modelProvider,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Medium,
                        color = Slate500,
                        modifier = Modifier.padding(horizontal = 4.dp, vertical = 1.dp)
                    )
                }
            }
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = "${numberFormat.format(item.totalTokens)} tok",
                fontFamily = FontFamily.Monospace,
                fontWeight = FontWeight.Bold,
                fontSize = 12.sp,
                color = PrimaryIndigo
            )
        }
        Spacer(modifier = Modifier.height(4.dp))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = item.agentId,
                fontFamily = FontFamily.Monospace,
                fontSize = 11.sp,
                color = Slate600,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f, fill = false)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = "In: ${numberFormat.format(item.inputTokens)} · Out: ${numberFormat.format(item.outputTokens)}",
                fontFamily = FontFamily.Monospace,
                fontSize = 10.sp,
                color = Slate500
            )
        }
        Spacer(modifier = Modifier.height(2.dp))
        Text(
            text = formatTimestamp(item.timestamp),
            fontFamily = FontFamily.Monospace,
            fontSize = 10.sp,
            color = Slate400
        )
    }
}

private fun formatNumber(value: Long): String {
    return NumberFormat.getIntegerInstance().format(value)
}

private fun formatTimestamp(isoString: String): String {
    return try {
        if (isoString.length >= 16) {
            val datePart = isoString.substring(5, 10)
            val timePart = isoString.substring(11, 16)
            "$datePart $timePart"
        } else {
            isoString
        }
    } catch (_: Exception) {
        isoString
    }
}

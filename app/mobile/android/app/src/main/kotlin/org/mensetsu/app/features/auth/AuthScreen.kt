package org.mensetsu.app.features.auth

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.mensetsu.app.core.network.NetworkManager
import org.mensetsu.app.core.tags.AppTestTags
import org.mensetsu.contracts.models.Login2faRequestDto
import org.mensetsu.contracts.models.LoginRequestDto
import org.mensetsu.contracts.models.RegisterRequestDto
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AuthScreen(
    networkManager: NetworkManager,
    onAuthSuccess: () -> Unit,
    onChangeServer: () -> Unit = {}
) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }

    var isLoading by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }

    // 2FA state
    var show2faDialog by remember { mutableStateOf(false) }
    var temp2faToken by remember { mutableStateOf("") }
    var code2fa by remember { mutableStateOf("") }

    val coroutineScope = rememberCoroutineScope()

    Scaffold(
        modifier = Modifier.testTag(AppTestTags.AUTH_SCREEN),
        topBar = {
            TopAppBar(
                title = { Text("Mensetsu Interview Studio") },
                actions = {
                    IconButton(
                        onClick = onChangeServer,
                        modifier = Modifier.testTag(AppTestTags.BUTTON_SETTINGS)
                    ) {
                        Icon(Icons.Default.Settings, contentDescription = "Server Settings")
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(24.dp)
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = if (isRegisterTab) "Create New Account" else "Welcome Back",
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = "Autonomous Technical Interview Intelligence & Practice",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Spacer(modifier = Modifier.height(24.dp))

            TabRow(
                selectedTabIndex = if (isRegisterTab) 1 else 0,
                modifier = Modifier.fillMaxWidth()
            ) {
                Tab(
                    selected = !isRegisterTab,
                    onClick = {
                        isRegisterTab = false
                        errorMessage = null
                    },
                    text = { Text("Log In") },
                    modifier = Modifier.testTag(AppTestTags.AUTH_TAB_LOGIN)
                )
                Tab(
                    selected = isRegisterTab,
                    onClick = {
                        isRegisterTab = true
                        errorMessage = null
                    },
                    text = { Text("Register") },
                    modifier = Modifier.testTag(AppTestTags.AUTH_TAB_REGISTER)
                )
            }

            Spacer(modifier = Modifier.height(24.dp))

            if (errorMessage != null) {
                Surface(
                    color = MaterialTheme.colorScheme.errorContainer,
                    shape = MaterialTheme.shapes.small,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = errorMessage ?: "",
                        color = MaterialTheme.colorScheme.onErrorContainer,
                        style = MaterialTheme.typography.bodyMedium,
                        modifier = Modifier
                            .padding(12.dp)
                            .testTag(AppTestTags.AUTH_ERROR_TEXT)
                    )
                }
                Spacer(modifier = Modifier.height(16.dp))
            }

            if (isRegisterTab) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Full Name") },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .testTag(AppTestTags.AUTH_INPUT_NAME)
                )
                Spacer(modifier = Modifier.height(12.dp))
            }

            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text("Email Address") },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                singleLine = true,
                modifier = Modifier
                    .fillMaxWidth()
                    .testTag(AppTestTags.AUTH_INPUT_EMAIL)
            )

            Spacer(modifier = Modifier.height(12.dp))

            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text("Password") },
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                singleLine = true,
                modifier = Modifier
                    .fillMaxWidth()
                    .testTag(AppTestTags.AUTH_INPUT_PASSWORD)
            )

            Spacer(modifier = Modifier.height(24.dp))

            Button(
                onClick = {
                    errorMessage = null
                    isLoading = true
                    coroutineScope.launch {
                        try {
                            if (isRegisterTab) {
                                val response = withContext(Dispatchers.IO) {
                                    networkManager.getAuthApi().register(
                                        RegisterRequestDto(
                                            email = email.trim(),
                                            password = password,
                                            name = name.trim()
                                        )
                                    )
                                }
                                networkManager.setAccessToken(response.accessToken)
                                networkManager.saveRefreshToken(response.refreshToken)
                                isLoading = false
                                onAuthSuccess()
                            } else {
                                val response = withContext(Dispatchers.IO) {
                                    networkManager.getAuthApi().login(
                                        LoginRequestDto(
                                            email = email.trim(),
                                            password = password
                                        )
                                    )
                                }
                                val tempToken = response.tempToken
                                if (response.requires2fa == true && !tempToken.isNullOrBlank()) {
                                    temp2faToken = tempToken
                                    show2faDialog = true
                                    isLoading = false
                                } else {
                                    networkManager.setAccessToken(response.accessToken)
                                    networkManager.saveRefreshToken(response.refreshToken)
                                    isLoading = false
                                    onAuthSuccess()
                                }
                            }
                        } catch (e: Exception) {
                            isLoading = false
                            errorMessage = e.message ?: "Authentication failed. Please verify credentials."
                        }
                    }
                },
                enabled = !isLoading && email.isNotBlank() && password.isNotBlank() && (!isRegisterTab || name.isNotBlank()),
                modifier = Modifier
                    .fillMaxWidth()
                    .height(50.dp)
                    .testTag(AppTestTags.AUTH_BUTTON_SUBMIT)
            ) {
                if (isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(24.dp),
                        color = MaterialTheme.colorScheme.onPrimary
                    )
                } else {
                    Text(if (isRegisterTab) "Sign Up" else "Sign In")
                }
            }
        }
    }

    if (show2faDialog) {
        AlertDialog(
            onDismissRequest = {
                show2faDialog = false
                code2fa = ""
            },
            title = { Text("Two-Factor Authentication") },
            text = {
                Column {
                    Text("Enter the 6-digit TOTP verification code from your authenticator app:")
                    Spacer(modifier = Modifier.height(16.dp))
                    OutlinedTextField(
                        value = code2fa,
                        onValueChange = { code2fa = it },
                        label = { Text("2FA Code") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier
                            .fillMaxWidth()
                            .testTag(AppTestTags.INPUT_2FA_CODE)
                    )
                }
            },
            confirmButton = {
                Button(
                    onClick = {
                        isLoading = true
                        show2faDialog = false
                        coroutineScope.launch {
                            try {
                                val response = withContext(Dispatchers.IO) {
                                    networkManager.getAuthApi().login2fa(
                                        Login2faRequestDto(
                                            code = code2fa.trim(),
                                            tempToken = temp2faToken
                                        )
                                    )
                                }
                                networkManager.setAccessToken(response.accessToken)
                                networkManager.saveRefreshToken(response.refreshToken)
                                isLoading = false
                                onAuthSuccess()
                            } catch (e: Exception) {
                                isLoading = false
                                errorMessage = e.message ?: "Invalid 2FA code."
                            }
                        }
                    },
                    enabled = code2fa.length >= 6,
                    modifier = Modifier.testTag(AppTestTags.BUTTON_2FA_VERIFY)
                ) {
                    Text("Verify")
                }
            },
            dismissButton = {
                TextButton(onClick = { show2faDialog = false }) {
                    Text("Cancel")
                }
            },
            modifier = Modifier.testTag(AppTestTags.DIALOG_2FA)
        )
    }
}

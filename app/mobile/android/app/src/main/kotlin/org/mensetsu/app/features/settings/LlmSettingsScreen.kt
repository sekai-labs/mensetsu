package org.mensetsu.app.features.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.mensetsu.app.core.network.EndpointValidator
import org.mensetsu.app.core.network.NetworkManager
import org.mensetsu.app.core.tags.AppTestTags
import org.mensetsu.contracts.models.LlmConfigItemDto
import org.mensetsu.contracts.models.UpsertLlmConfigRequestDto

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LlmSettingsScreen(
    networkManager: NetworkManager,
    onBack: (() -> Unit)? = null,
    onEndpointChanged: (() -> Unit)? = null
) {
    var settings by remember { mutableStateOf<List<LlmConfigItemDto>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var editingConfig by remember { mutableStateOf<LlmConfigItemDto?>(null) }

    var endpointInput by remember { mutableStateOf(networkManager.getBaseUrl()) }
    var endpointError by remember { mutableStateOf<String?>(null) }
    var endpointSuccessMessage by remember { mutableStateOf<String?>(null) }

    val coroutineScope = rememberCoroutineScope()
    fun loadSettings() {
        isLoading = true
        errorMessage = null
        coroutineScope.launch {
            try {
                val list = withContext(Dispatchers.IO) {
                    networkManager.getSettingsApi().getLlmSettings()
                }
                settings = list
                isLoading = false
            } catch (e: Exception) {
                isLoading = false
                errorMessage = e.message ?: "Failed to load LLM settings."
            }
        }
    }

    LaunchedEffect(Unit) {
        loadSettings()
    }

    Scaffold(
        modifier = Modifier.testTag(AppTestTags.SETTINGS_SCREEN),
        topBar = {
            TopAppBar(
                title = { Text("LLM Settings") },
                navigationIcon = {
                    if (onBack != null) {
                        IconButton(onClick = onBack) {
                            Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                        }
                    }
                }
            )
        }
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            item {
                Text(
                    text = "Server Configuration",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "Configured self-hosted Rust API endpoint. Changing this clears credentials and resets your session.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(modifier = Modifier.height(8.dp))
                Card(
                    modifier = Modifier.fillMaxWidth()
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
                                color = MaterialTheme.colorScheme.primary,
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
                Spacer(modifier = Modifier.height(16.dp))
                Divider()
                Spacer(modifier = Modifier.height(16.dp))
                Text(
                    text = "LLM Provider Configuration",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "Configure underlying LLM provider keys for research synthesis and interview questions.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(modifier = Modifier.height(8.dp))
            }

            when {
                isLoading -> {
                    item {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 32.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            CircularProgressIndicator()
                        }
                    }
                }
                errorMessage != null -> {
                    item {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                Text(
                                    text = errorMessage ?: "Error loading settings",
                                     color = MaterialTheme.colorScheme.error
                                 )
                                 Spacer(modifier = Modifier.height(16.dp))
                                 Button(onClick = { loadSettings() }) {
                                     Text("Retry")
                                 }
                             }
                         }
                     }
                 }
                 else -> {
                     items(settings, key = { it.provider }) { config ->
                         Card(
                             modifier = Modifier
                                 .fillMaxWidth()
                                 .testTag("${AppTestTags.SETTINGS_PROVIDER_PREFIX}${config.provider}")
                         ) {
                             Row(
                                 modifier = Modifier
                                     .fillMaxWidth()
                                     .padding(16.dp),
                                 horizontalArrangement = Arrangement.SpaceBetween,
                                 verticalAlignment = Alignment.CenterVertically
                             ) {
                                 Column(modifier = Modifier.weight(1f)) {
                                     Text(
                                         text = config.provider.uppercase(),
                                         style = MaterialTheme.typography.titleMedium,
                                         fontWeight = FontWeight.Bold
                                     )
                                     Spacer(modifier = Modifier.height(4.dp))
                                     if (config.hasKey) {
                                         Row(verticalAlignment = Alignment.CenterVertically) {
                                             Icon(
                                                 Icons.Default.Check,
                                                 contentDescription = "Active",
                                                 tint = Color(0xFF059669),
                                                 modifier = Modifier.size(16.dp)
                                             )
                                             Spacer(modifier = Modifier.width(4.dp))
                                             Text(
                                                 text = config.maskedKey ?: "Key Configured",
                                                 style = MaterialTheme.typography.bodySmall,
                                                 color = Color(0xFF059669)
                                             )
                                         }
                                     } else {
                                         Text(
                                             text = "Not Configured",
                                             style = MaterialTheme.typography.bodySmall,
                                             color = MaterialTheme.colorScheme.outline
                                         )
                                     }
                                     if (!config.modelChat.isNullOrBlank()) {
                                         Spacer(modifier = Modifier.height(2.dp))
                                         Text(
                                             text = "Chat Model: ${config.modelChat}",
                                             style = MaterialTheme.typography.labelSmall,
                                             color = MaterialTheme.colorScheme.onSurfaceVariant
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
                                         modifier = Modifier.size(16.dp)
                                     )
                                     Spacer(modifier = Modifier.width(6.dp))
                                     Text("Set Key")
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

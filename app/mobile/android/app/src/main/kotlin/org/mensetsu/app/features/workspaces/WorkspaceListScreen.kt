package org.mensetsu.app.features.workspaces

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ExitToApp
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.mensetsu.app.core.network.NetworkManager
import org.mensetsu.app.core.network.SelectedPdf
import org.mensetsu.app.core.tags.AppTestTags
import org.mensetsu.contracts.models.CreateWorkspaceRequestDto
import org.mensetsu.contracts.models.WorkspaceResponseDto
import org.mensetsu.contracts.models.WorkspaceStatus

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WorkspaceListScreen(
    networkManager: NetworkManager,
    onNavigateToDetail: (workspaceId: String) -> Unit,
    onNavigateToSettings: () -> Unit,
    onLogout: () -> Unit
) {
    var workspaces by remember { mutableStateOf<List<WorkspaceResponseDto>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var selectedFilter by remember { mutableStateOf("all") } // all, ready, processing

    var showCreateDialog by remember { mutableStateOf(false) }

    val coroutineScope = rememberCoroutineScope()

    fun loadWorkspaces() {
        isLoading = true
        errorMessage = null
        coroutineScope.launch {
            try {
                val list = withContext(Dispatchers.IO) {
                    networkManager.getWorkspacesApi().listWorkspaces()
                }
                workspaces = list
                isLoading = false
            } catch (e: Exception) {
                isLoading = false
                errorMessage = e.message ?: "Failed to load workspaces from backend."
            }
        }
    }

    LaunchedEffect(Unit) {
        loadWorkspaces()
    }

    val filteredList = remember(workspaces, selectedFilter) {
        when (selectedFilter) {
            "ready" -> workspaces.filter { it.status == WorkspaceStatus.ready }
            "processing" -> workspaces.filter {
                it.status == WorkspaceStatus.running ||
                        it.status == WorkspaceStatus.pending ||
                        it.status == WorkspaceStatus.queued ||
                        it.status == WorkspaceStatus.dispatched
            }
            else -> workspaces
        }
    }

    Scaffold(
        modifier = Modifier.testTag(AppTestTags.WORKSPACE_LIST_SCREEN),
        topBar = {
            TopAppBar(
                title = { Text("Workspaces") },
                actions = {
                    IconButton(
                        onClick = { loadWorkspaces() },
                        modifier = Modifier.testTag(AppTestTags.WORKSPACE_LIST_RETRY_BUTTON)
                    ) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                    IconButton(
                        onClick = onNavigateToSettings,
                        modifier = Modifier.testTag(AppTestTags.BUTTON_SETTINGS)
                    ) {
                        Icon(Icons.Default.Settings, contentDescription = "LLM Settings")
                    }
                    IconButton(
                        onClick = {
                            coroutineScope.launch {
                                try {
                                    withContext(Dispatchers.IO) {
                                        networkManager.getAuthApi().logout()
                                    }
                                } catch (_: Exception) {}
                                networkManager.clearSession()
                                onLogout()
                            }
                        },
                        modifier = Modifier.testTag(AppTestTags.BUTTON_LOGOUT)
                    ) {
                        Icon(Icons.Default.ExitToApp, contentDescription = "Logout")
                    }
                }
            )
        },
        floatingActionButton = {
            FloatingActionButton(
                onClick = { showCreateDialog = true },
                modifier = Modifier.testTag(AppTestTags.BUTTON_NEW_WORKSPACE)
            ) {
                Icon(Icons.Default.Add, contentDescription = "New Workspace")
            }
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            // Filter Tabs
            TabRow(
                selectedTabIndex = when (selectedFilter) {
                    "ready" -> 1
                    "processing" -> 2
                    else -> 0
                },
                modifier = Modifier.fillMaxWidth()
            ) {
                Tab(
                    selected = selectedFilter == "all",
                    onClick = { selectedFilter = "all" },
                    text = { Text("All (${workspaces.size})") },
                    modifier = Modifier.testTag(AppTestTags.FILTER_TAB_ALL)
                )
                Tab(
                    selected = selectedFilter == "ready",
                    onClick = { selectedFilter = "ready" },
                    text = { Text("Ready") },
                    modifier = Modifier.testTag(AppTestTags.FILTER_TAB_READY)
                )
                Tab(
                    selected = selectedFilter == "processing",
                    onClick = { selectedFilter = "processing" },
                    text = { Text("Processing") },
                    modifier = Modifier.testTag(AppTestTags.FILTER_TAB_PROCESSING)
                )
            }

            when {
                isLoading -> {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .testTag(AppTestTags.WORKSPACE_LIST_LOADING),
                        contentAlignment = Alignment.Center
                    ) {
                        CircularProgressIndicator()
                    }
                }
                errorMessage != null -> {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(24.dp)
                            .testTag(AppTestTags.WORKSPACE_LIST_ERROR),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text(
                                text = errorMessage ?: "Error loading workspaces",
                                color = MaterialTheme.colorScheme.error,
                                style = MaterialTheme.typography.bodyLarge
                            )
                            Spacer(modifier = Modifier.height(16.dp))
                            Button(onClick = { loadWorkspaces() }) {
                                Text("Retry")
                            }
                        }
                    }
                }
                filteredList.isEmpty() -> {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .testTag(AppTestTags.WORKSPACE_LIST_EMPTY),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text(
                                text = "No workspaces found",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = "Tap '+' to start a new interview preparation workspace.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.outline
                            )
                        }
                    }
                }
                else -> {
                    LazyColumn(
                        modifier = Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(16.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        items(filteredList, key = { it.id }) { ws ->
                            WorkspaceCard(
                                workspace = ws,
                                onClick = { onNavigateToDetail(ws.id) }
                            )
                        }
                    }
                }
            }
        }
    }

    if (showCreateDialog) {
        CreateWorkspaceDialog(
            networkManager = networkManager,
            onDismiss = { showCreateDialog = false },
            onCreated = { newWs ->
                showCreateDialog = false
                loadWorkspaces()
                onNavigateToDetail(newWs.id)
            }
        )
    }
}

@Composable
fun WorkspaceCard(
    workspace: WorkspaceResponseDto,
    onClick: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .testTag("${AppTestTags.WORKSPACE_ITEM_PREFIX}${workspace.id}")
            .clickable { onClick() },
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = workspace.companyName,
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold
                )
                StatusChip(status = workspace.status)
            }
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = workspace.roleTitle,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(12.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Text(
                    text = "Progress: ${workspace.progressPercent}%",
                    style = MaterialTheme.typography.labelSmall
                )
                Text(
                    text = "Phase: ${workspace.currentPhase.value}",
                    style = MaterialTheme.typography.labelSmall
                )
            }
            Spacer(modifier = Modifier.height(6.dp))
            LinearProgressIndicator(
                progress = { (workspace.progressPercent.coerceIn(0, 100)) / 100f },
                modifier = Modifier.fillMaxWidth()
            )
        }
    }
}

@Composable
fun StatusChip(status: WorkspaceStatus) {
    val (bgColor, textColor, label) = when (status) {
        WorkspaceStatus.ready -> Triple(Color(0xFFECFDF5), Color(0xFF059669), "READY")
        WorkspaceStatus.running, WorkspaceStatus.dispatched -> Triple(Color(0xFFEEF2FF), Color(0xFF4F46E5), "RUNNING")
        WorkspaceStatus.failed -> Triple(Color(0xFFFFF1F2), Color(0xFFE11D48), "FAILED")
        else -> Triple(Color(0xFFFFFBEB), Color(0xFFD97706), status.value.uppercase())
    }

    Surface(
        color = bgColor,
        shape = MaterialTheme.shapes.extraSmall
    ) {
        Text(
            text = label,
            color = textColor,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp)
        )
    }
}

@Composable
fun CreateWorkspaceDialog(
    networkManager: NetworkManager,
    onDismiss: () -> Unit,
    onCreated: (WorkspaceResponseDto) -> Unit
) {
    var company by remember { mutableStateOf("") }
    var role by remember { mutableStateOf("") }
    var resumeText by remember { mutableStateOf("") }
    var jobDesc by remember { mutableStateOf("") }
    var usePdf by remember { mutableStateOf(false) }
    var selectedPdf by remember { mutableStateOf<SelectedPdf?>(null) }

    var isSubmitting by remember { mutableStateOf(false) }
    var dialogError by remember { mutableStateOf<String?>(null) }

    val coroutineScope = rememberCoroutineScope()
    val context = LocalContext.current
    val pdfPicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) {
            coroutineScope.launch {
                try {
                    selectedPdf = withContext(Dispatchers.IO) {
                        networkManager.inspectPdf(context.contentResolver, uri)
                    }
                    dialogError = null
                } catch (e: Exception) {
                    dialogError = e.message ?: "Cannot read the selected PDF."
                }
            }
        }
    }

    AlertDialog(
        onDismissRequest = { if (!isSubmitting) onDismiss() },
        title = { Text("New Workspace") },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(vertical = 4.dp)
            ) {
                Text(
                    text = "Create an evidence-backed interview workspace with résumé text or a PDF and job description.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(modifier = Modifier.height(12.dp))

                if (dialogError != null) {
                    Text(
                        text = dialogError ?: "",
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.testTag(AppTestTags.CREATE_WORKSPACE_ERROR)
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                }

                OutlinedTextField(
                    value = company,
                    onValueChange = { company = it },
                    label = { Text("Company Name *") },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .testTag(AppTestTags.INPUT_CREATE_COMPANY)
                )
                Spacer(modifier = Modifier.height(8.dp))

                OutlinedTextField(
                    value = role,
                    onValueChange = { role = it },
                    label = { Text("Role Title *") },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .testTag(AppTestTags.INPUT_CREATE_ROLE)
                )
                Spacer(modifier = Modifier.height(8.dp))

                Row(verticalAlignment = Alignment.CenterVertically) {
                    FilterChip(
                        selected = !usePdf,
                        onClick = { usePdf = false },
                        label = { Text("Paste text") },
                        enabled = !isSubmitting,
                        modifier = Modifier.testTag(AppTestTags.BUTTON_CREATE_TEXT_MODE)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    FilterChip(
                        selected = usePdf,
                        onClick = { usePdf = true },
                        label = { Text("Upload PDF") },
                        enabled = !isSubmitting,
                        modifier = Modifier.testTag(AppTestTags.BUTTON_CREATE_PDF_MODE)
                    )
                }
                if (usePdf) {
                    Button(
                        onClick = { pdfPicker.launch(arrayOf("application/pdf")) },
                        enabled = !isSubmitting,
                        modifier = Modifier.testTag(AppTestTags.BUTTON_CREATE_PICK_PDF)
                    ) {
                        Text("Choose PDF")
                    }
                    Text(
                        selectedPdf?.filename ?: "PDF up to 5 MiB",
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.testTag(AppTestTags.CREATE_PDF_FILENAME)
                    )
                } else {
                    OutlinedTextField(
                        value = resumeText,
                        onValueChange = { resumeText = it },
                        label = { Text("Résumé Text") },
                        minLines = 3,
                        maxLines = 5,
                        modifier = Modifier
                            .fillMaxWidth()
                            .testTag(AppTestTags.INPUT_CREATE_RESUME_TEXT)
                    )
                }
                Spacer(modifier = Modifier.height(8.dp))

                OutlinedTextField(
                    value = jobDesc,
                    onValueChange = { jobDesc = it },
                    label = { Text("Job Description") },
                    minLines = 3,
                    maxLines = 5,
                    modifier = Modifier
                        .fillMaxWidth()
                        .testTag(AppTestTags.INPUT_CREATE_JOB_DESC)
                )
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    if (isSubmitting) return@Button
                    dialogError = null
                    isSubmitting = true
                    coroutineScope.launch {
                        try {
                            val companyName = company.trim()
                            val roleTitle = role.trim()
                            val description = jobDesc.trim()
                            val created = withContext(Dispatchers.IO) {
                                if (usePdf) {
                                    val document = selectedPdf ?: error("Choose a PDF résumé.")
                                    networkManager.createWorkspaceWithPdf(
                                        context.contentResolver, document,
                                        companyName, roleTitle, description
                                    )
                                } else {
                                    networkManager.getWorkspacesApi().createWorkspace(
                                        CreateWorkspaceRequestDto(
                                            companyName = companyName,
                                            roleTitle = roleTitle,
                                            resumeText = resumeText.trim().ifEmpty { null },
                                            jobDescription = description.ifEmpty { null }
                                        )
                                    )
                                }
                            }
                            isSubmitting = false
                            onCreated(created)
                        } catch (e: Exception) {
                            isSubmitting = false
                            dialogError = e.message ?: "Failed to create workspace."
                        }
                    }
                },
                enabled = !isSubmitting && company.isNotBlank() && role.isNotBlank() && (!usePdf || selectedPdf != null),
                modifier = Modifier.testTag(AppTestTags.BUTTON_CREATE_SUBMIT)
            ) {
                if (isSubmitting) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        color = MaterialTheme.colorScheme.onPrimary
                    )
                } else {
                    Text("Create")
                }
            }
        },
        dismissButton = {
            TextButton(
                onClick = onDismiss,
                enabled = !isSubmitting,
                modifier = Modifier.testTag(AppTestTags.BUTTON_CREATE_CANCEL)
            ) {
                Text("Cancel")
            }
        },
        modifier = Modifier.testTag(AppTestTags.DIALOG_CREATE_WORKSPACE)
    )
}

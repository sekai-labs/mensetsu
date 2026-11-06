package org.mensetsu.app.features.workspaces

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.mensetsu.app.core.network.NetworkManager
import org.mensetsu.app.core.tags.AppTestTags
import org.mensetsu.app.features.company.CompanyDossierTab
import org.mensetsu.app.features.learning.LearningPathTab
import org.mensetsu.app.features.match.MatchAnalysisTab
import org.mensetsu.app.features.questions.QuestionBookTab
import org.mensetsu.app.features.settings.LlmSettingsScreen
import org.mensetsu.contracts.infrastructure.Serializer
import org.mensetsu.contracts.models.CompanyResearch
import org.mensetsu.contracts.models.InterviewQuestion
import org.mensetsu.contracts.models.LearningUnit
import org.mensetsu.contracts.models.MatchAnalysis
import org.mensetsu.contracts.models.WorkspaceResponseDto

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WorkspaceDetailScreen(
    workspaceId: String,
    networkManager: NetworkManager,
    onBack: () -> Unit
) {
    var workspace by remember { mutableStateOf<WorkspaceResponseDto?>(null) }
    var match by remember { mutableStateOf<MatchAnalysis?>(null) }
    var company by remember { mutableStateOf<CompanyResearch?>(null) }
    var questions by remember { mutableStateOf<List<InterviewQuestion>>(emptyList()) }
    var learning by remember { mutableStateOf<List<LearningUnit>>(emptyList()) }

    var isLoading by remember { mutableStateOf(true) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var selectedTabIndex by remember { mutableStateOf(0) }

    val coroutineScope = rememberCoroutineScope()

    val moshi = remember {
        Moshi.Builder()
            .add(com.squareup.moshi.adapters.Rfc3339DateJsonAdapter())
            .addLast(KotlinJsonAdapterFactory())
            .build()
    }

    fun loadAllData() {
        isLoading = true
        errorMessage = null
        coroutineScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    val ws = networkManager.getWorkspacesApi().getWorkspace(workspaceId)
                    workspace = ws

                    try {
                        val rawMatch = networkManager.getLearningHandlersApi().getWorkspaceMatch(workspaceId)
                        val matchJson = Serializer.moshi.adapter(Any::class.java).toJson(rawMatch)
                        match = Serializer.moshi.adapter(MatchAnalysis::class.java).fromJson(matchJson)
                    } catch (_: Exception) {
                        match = null
                    }

                    try {
                        val rawCompany = networkManager.getResearchHandlersApi().getWorkspaceCompany(workspaceId)
                        val compJson = Serializer.moshi.adapter(Any::class.java).toJson(rawCompany)
                        company = Serializer.moshi.adapter(CompanyResearch::class.java).fromJson(compJson)
                    } catch (_: Exception) {
                        company = null
                    }

                    try {
                        questions = networkManager.getLearningHandlersApi().getWorkspaceQuestions(workspaceId)
                    } catch (_: Exception) {
                        questions = emptyList()
                    }

                    try {
                        learning = networkManager.getLearningHandlersApi().getWorkspaceLearning(workspaceId)
                    } catch (_: Exception) {
                        learning = emptyList()
                    }
                }
                isLoading = false
            } catch (e: Exception) {
                isLoading = false
                errorMessage = e.message ?: "Failed to load workspace details."
            }
        }
    }

    LaunchedEffect(workspaceId) {
        loadAllData()
    }

    fun toggleLesson(lessonId: String) {
        coroutineScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    networkManager.getWorkspacesApi().toggleLearningUnitCompleted(workspaceId, lessonId)
                }
                learning = learning.map { l ->
                    if (l.id == lessonId) {
                        l.copy(completed = !(l.completed ?: false))
                    } else {
                        l
                    }
                }
            } catch (e: Exception) {
                loadAllData()
            }
        }
    }

    Scaffold(
        modifier = Modifier.testTag(AppTestTags.WORKSPACE_DETAIL_SCREEN),
        topBar = {
            TopAppBar(
                title = { Text(workspace?.companyName ?: "Workspace Detail") },
                navigationIcon = {
                    IconButton(
                        onClick = onBack,
                        modifier = Modifier.testTag(AppTestTags.BUTTON_DETAIL_BACK)
                    ) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
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
            when {
                isLoading -> {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .testTag(AppTestTags.DETAIL_LOADING),
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
                            .testTag(AppTestTags.DETAIL_ERROR),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text(
                                text = errorMessage ?: "Error loading workspace details",
                                color = MaterialTheme.colorScheme.error,
                                style = MaterialTheme.typography.bodyLarge
                            )
                            Spacer(modifier = Modifier.height(16.dp))
                            Button(onClick = { loadAllData() }) {
                                Text("Retry")
                            }
                        }
                    }
                }
                workspace != null -> {
                    ScrollableTabRow(
                        selectedTabIndex = selectedTabIndex,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Tab(
                            selected = selectedTabIndex == 0,
                            onClick = { selectedTabIndex = 0 },
                            text = { Text("Overview") },
                            modifier = Modifier.testTag(AppTestTags.DETAIL_TAB_OVERVIEW)
                        )
                        Tab(
                            selected = selectedTabIndex == 1,
                            onClick = { selectedTabIndex = 1 },
                            text = { Text("Match") },
                            modifier = Modifier.testTag(AppTestTags.DETAIL_TAB_MATCH)
                        )
                        Tab(
                            selected = selectedTabIndex == 2,
                            onClick = { selectedTabIndex = 2 },
                            text = { Text("Company") },
                            modifier = Modifier.testTag(AppTestTags.DETAIL_TAB_COMPANY)
                        )
                        Tab(
                            selected = selectedTabIndex == 3,
                            onClick = { selectedTabIndex = 3 },
                            text = { Text("Learning") },
                            modifier = Modifier.testTag(AppTestTags.DETAIL_TAB_LEARNING)
                        )
                        Tab(
                            selected = selectedTabIndex == 4,
                            onClick = { selectedTabIndex = 4 },
                            text = { Text("Questions") },
                            modifier = Modifier.testTag(AppTestTags.DETAIL_TAB_QUESTIONS)
                        )
                        Tab(
                            selected = selectedTabIndex == 5,
                            onClick = { selectedTabIndex = 5 },
                            text = { Text("LLM") },
                            modifier = Modifier.testTag(AppTestTags.DETAIL_TAB_SETTINGS)
                        )
                    }

                    Box(modifier = Modifier.fillMaxSize()) {
                        when (selectedTabIndex) {
                            0 -> WorkspaceOverviewTab(
                                workspace = workspace!!,
                                match = match,
                                company = company,
                                questions = questions,
                                learning = learning,
                                onSelectTab = { selectedTabIndex = it }
                            )
                            1 -> MatchAnalysisTab(match = match)
                            2 -> CompanyDossierTab(company = company)
                            3 -> LearningPathTab(
                                lessons = learning,
                                onToggleCompleted = { toggleLesson(it) }
                            )
                            4 -> QuestionBookTab(questions = questions)
                            5 -> LlmSettingsScreen(networkManager = networkManager)
                        }
                    }
                }
            }
        }
    }
}

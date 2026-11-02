package org.mensetsu.app.features.workspaces

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import org.mensetsu.contracts.models.*

@Composable
fun WorkspaceOverviewTab(
    workspace: WorkspaceResponseDto,
    match: MatchAnalysis?,
    company: CompanyResearch?,
    questions: List<InterviewQuestion>,
    learning: List<LearningUnit>,
    onSelectTab: (Int) -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Workspace Status Card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Preparation Status",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = "Current Objective: ${workspace.currentObjective ?: "Comprehensive interview readiness analysis"}",
                    style = MaterialTheme.typography.bodyMedium
                )
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = "Pipeline Phase: ${workspace.currentPhase.value} (${workspace.progressPercent}%)",
                    style = MaterialTheme.typography.labelSmall
                )
            }
        }

        // Summary Metric Cards
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            MetricCard(
                title = "Match Ratio",
                value = if (match != null) "${(match.overallMatchRatio * 100).toInt()}%" else "--",
                subtitle = "Resume/JD Alignment",
                modifier = Modifier.weight(1f),
                onClick = { onSelectTab(1) }
            )
            MetricCard(
                title = "Company Dossier",
                value = if (company != null) "${company.sources.size} Sources" else "--",
                subtitle = "Intel & Culture",
                modifier = Modifier.weight(1f),
                onClick = { onSelectTab(2) }
            )
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            val completedCount = learning.count { it.completed == true }
            MetricCard(
                title = "Learning Units",
                value = "$completedCount / ${learning.size}",
                subtitle = "Units Mastered",
                modifier = Modifier.weight(1f),
                onClick = { onSelectTab(3) }
            )
            MetricCard(
                title = "Curated Questions",
                value = "${questions.size} In Book",
                subtitle = "Targeted Rubrics",
                modifier = Modifier.weight(1f),
                onClick = { onSelectTab(4) }
            )
        }
    }
}

@Composable
fun MetricCard(
    title: String,
    value: String,
    subtitle: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Card(
        onClick = onClick,
        modifier = modifier
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = value,
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.primary
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = subtitle,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.outline
            )
        }
    }
}

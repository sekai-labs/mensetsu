package org.mensetsu.app.features.learning

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.outlined.Circle
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import org.mensetsu.app.core.tags.AppTestTags
import org.mensetsu.contracts.models.LearningUnit

@Composable
fun LearningPathTab(
    lessons: List<LearningUnit>,
    onToggleCompleted: (lessonId: String) -> Unit
) {
    if (lessons.isEmpty()) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "No learning units generated yet.",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        return
    }

    var selectedLessonId by remember(lessons) {
        mutableStateOf(lessons.firstOrNull()?.id)
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item {
            Text(
                text = "Personalized Learning Path",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(4.dp))
            val completedCount = lessons.count { it.completed == true }
            Text(
                text = "$completedCount of ${lessons.size} units mastered",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(8.dp))
        }

        items(lessons, key = { it.id }) { lesson ->
            val isCompleted = lesson.completed == true
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .testTag("${AppTestTags.LEARNING_UNIT_PREFIX}${lesson.id}")
                    .clickable { selectedLessonId = lesson.id },
                colors = CardDefaults.cardColors(
                    containerColor = if (lesson.id == selectedLessonId)
                        MaterialTheme.colorScheme.surfaceVariant
                    else
                        MaterialTheme.colorScheme.surface
                )
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IconButton(
                        onClick = { onToggleCompleted(lesson.id) },
                        modifier = Modifier.testTag("${AppTestTags.LEARNING_TOGGLE_PREFIX}${lesson.id}")
                    ) {
                        if (isCompleted) {
                            Icon(
                                Icons.Filled.CheckCircle,
                                contentDescription = "Completed",
                                tint = Color(0xFF059669)
                            )
                        } else {
                            Icon(
                                Icons.Outlined.Circle,
                                contentDescription = "Mark Complete",
                                tint = MaterialTheme.colorScheme.outline
                            )
                        }
                    }

                    Spacer(modifier = Modifier.width(8.dp))

                    Column(modifier = Modifier.weight(1f)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween
                        ) {
                            Text(
                                text = lesson.title,
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.SemiBold
                            )
                            Text(
                                text = "${lesson.estimatedMinutes}m",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.outline
                            )
                        }
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = lesson.summary,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        if (lesson.id == selectedLessonId && lesson.contentMarkdown.isNotBlank()) {
                            Spacer(modifier = Modifier.height(12.dp))
                            Divider()
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = lesson.contentMarkdown,
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }
                    }
                }
            }
        }
    }
}

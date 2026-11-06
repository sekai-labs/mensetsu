package org.mensetsu.app.features.questions

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import org.mensetsu.app.core.tags.AppTestTags
import org.mensetsu.contracts.models.InterviewQuestion
import org.mensetsu.contracts.models.QuestionDifficulty

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun QuestionBookTab(questions: List<InterviewQuestion>) {
    if (questions.isEmpty()) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "No questions in question book yet.",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        return
    }

    var selectedDifficulty by remember { mutableStateOf("all") }

    val filtered = remember(questions, selectedDifficulty) {
        if (selectedDifficulty == "all") {
            questions
        } else {
            questions.filter { it.difficulty.value.equals(selectedDifficulty, ignoreCase = true) }
        }
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            Column {
                Text(
                    text = "Curated Interview Question Book",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "High-leverage questions with interviewer intent, rubrics, and traps.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(modifier = Modifier.height(12.dp))

                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.testTag(AppTestTags.QUESTIONS_FILTER_DIFFICULTY)
                ) {
                    listOf("all" to "All", "easy" to "Easy", "medium" to "Medium", "hard" to "Hard").forEach { (key, label) ->
                        FilterChip(
                            selected = selectedDifficulty == key,
                            onClick = { selectedDifficulty = key },
                            label = { Text(label) }
                        )
                    }
                }
            }
        }

        items(filtered, key = { it.id }) { q ->
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .testTag("${AppTestTags.QUESTION_CARD_PREFIX}${q.id}")
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Surface(
                            color = when (q.difficulty) {
                                QuestionDifficulty.easy -> Color(0xFFECFDF5)
                                QuestionDifficulty.medium -> Color(0xFFFFFBEB)
                                QuestionDifficulty.hard -> Color(0xFFFFF1F2)
                            },
                            shape = MaterialTheme.shapes.extraSmall
                        ) {
                            Text(
                                text = q.difficulty.value.uppercase(),
                                color = when (q.difficulty) {
                                    QuestionDifficulty.easy -> Color(0xFF059669)
                                    QuestionDifficulty.medium -> Color(0xFFD97706)
                                    QuestionDifficulty.hard -> Color(0xFFE11D48)
                                },
                                style = MaterialTheme.typography.labelSmall,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp)
                            )
                        }

                        Text(
                            text = q.category.value,
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    Spacer(modifier = Modifier.height(8.dp))

                    Text(
                        text = q.question,
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold
                    )

                    Spacer(modifier = Modifier.height(8.dp))

                    Text(
                        text = "Interviewer Intent: ${q.interviewerIntent}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )

                    if (q.skillsTested.isNotEmpty()) {
                        Spacer(modifier = Modifier.height(8.dp))
                        FlowRow(
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                            verticalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            q.skillsTested.forEach { skill ->
                                AssistChip(
                                    onClick = {},
                                    label = { Text(skill, style = MaterialTheme.typography.labelSmall) }
                                )
                            }
                        }
                    }

                    if (q.strongAnswerRubric.isNotEmpty()) {
                        Spacer(modifier = Modifier.height(12.dp))
                        Divider()
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Strong Answer Rubric:",
                            style = MaterialTheme.typography.labelSmall,
                            fontWeight = FontWeight.Bold
                        )
                        q.strongAnswerRubric.forEach { rubric ->
                            Text(
                                text = "• $rubric",
                                style = MaterialTheme.typography.bodySmall,
                                modifier = Modifier.padding(vertical = 2.dp)
                            )
                        }
                    }
                }
            }
        }
    }
}

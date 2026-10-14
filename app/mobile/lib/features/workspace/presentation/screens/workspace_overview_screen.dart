import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../core/theme/app_typography.dart';
import '../../../../models.dart';
import '../widgets/metric_card.dart';

class WorkspaceOverviewScreen extends StatelessWidget {
  final WorkspaceDto workspace;
  final MatchAnalysisDto? match;
  final CompanyResearchDto? company;
  final List<InterviewQuestionDto> questions;
  final List<LearningUnitDto> lessons;
  final VoidCallback onNavigateToMatch;
  final VoidCallback onNavigateToCompany;
  final VoidCallback onNavigateToLearning;
  final VoidCallback onNavigateToQuestions;

  const WorkspaceOverviewScreen({
    super.key,
    required this.workspace,
    this.match,
    this.company,
    this.questions = const [],
    this.lessons = const [],
    required this.onNavigateToMatch,
    required this.onNavigateToCompany,
    required this.onNavigateToLearning,
    required this.onNavigateToQuestions,
  });

  @override
  Widget build(BuildContext context) {
    final matchPercent = match != null ? (match!.overallMatchRatio * 100).round() : 0;
    final gapsCount = match?.parsedGaps.length ?? 0;
    final completedLessons = lessons.where((l) => l.completed).length;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: Theme.of(context).cardTheme.color,
              border: Border.all(color: Theme.of(context).dividerColor),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                      decoration: BoxDecoration(
                        color: AppColors.emerald.withValues(alpha: 0.12),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        workspace.status.toUpperCase(),
                        style: AppTypography.badge(color: AppColors.emeraldDark),
                      ),
                    ),
                    Text(
                      'Target Readiness: ${workspace.progressPercent}%',
                      style: AppTypography.mono(context, fontSize: 11, color: AppColors.slate500),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                Text(
                  workspace.companyName,
                  style: AppTypography.h1(context),
                ),
                const SizedBox(height: 4),
                Text(
                  workspace.roleTitle,
                  style: AppTypography.body(context, color: AppColors.slate500),
                ),
                const SizedBox(height: 12),
                const Divider(height: 1),
                const SizedBox(height: 12),
                Text(
                  'Evidence-backed preparation pipeline synthesized from verified candidate records and company technical profile.',
                  style: AppTypography.body(context, color: AppColors.slate600),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),

          LayoutBuilder(
            builder: (context, constraints) {
              final isWide = constraints.maxWidth > 700;
              final cards = [
                MetricCard(
                  title: 'Candidate Match Ratio',
                  value: '$matchPercent%',
                  subtitle: 'Grounded against role requirements',
                  valueColor: AppColors.primary,
                  onTap: onNavigateToMatch,
                ),
                MetricCard(
                  title: 'Identified Skill Gaps',
                  value: '$gapsCount',
                  subtitle: 'Missing evidence items to bridge',
                  valueColor: AppColors.amberDark,
                  onTap: onNavigateToMatch,
                ),
                MetricCard(
                  title: 'Curated Question Rubrics',
                  value: '${questions.length}',
                  subtitle: 'Context-grounded interview book',
                  valueColor: AppColors.emeraldDark,
                  onTap: onNavigateToQuestions,
                ),
              ];

              if (isWide) {
                return Row(
                  children: cards.map((c) => Expanded(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 6), child: c))).toList(),
                );
              } else {
                return Column(
                  children: cards.map((c) => Padding(padding: const EdgeInsets.only(bottom: 12), child: c)).toList(),
                );
              }
            },
          ),
          const SizedBox(height: 20),

          if (company != null) ...[
            Container(
              padding: const EdgeInsets.all(20),
              decoration: BoxDecoration(
                color: Theme.of(context).cardTheme.color,
                border: Border.all(color: Theme.of(context).dividerColor),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Target Engineering Context', style: AppTypography.h2(context)),
                      TextButton(
                        onPressed: onNavigateToCompany,
                        child: const Text('View Full Dossier →', style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600)),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    company!.overview,
                    style: AppTypography.body(context),
                  ),
                  const SizedBox(height: 16),
                  Text('Primary Tech Stack:', style: AppTypography.h3(context)),
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 8,
                    runSpacing: 8,
                    children: company!.technologyStack.take(6).map((stack) {
                      return Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                        decoration: BoxDecoration(
                          color: Theme.of(context).colorScheme.surfaceContainerHighest,
                          borderRadius: BorderRadius.circular(4),
                          border: Border.all(color: Theme.of(context).dividerColor),
                        ),
                        child: Text(
                          stack,
                          style: AppTypography.mono(context, fontSize: 11),
                        ),
                      );
                    }).toList(),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 20),
          ],

          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: Theme.of(context).cardTheme.color,
              border: Border.all(color: Theme.of(context).dividerColor),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Targeted Curriculum', style: AppTypography.h2(context)),
                      const SizedBox(height: 4),
                      Text(
                        'Completed $completedLessons of ${lessons.length} core units.',
                        style: AppTypography.body(context, color: AppColors.slate500),
                      ),
                    ],
                  ),
                ),
                OutlinedButton(
                  onPressed: onNavigateToLearning,
                  child: const Text('Open Lessons'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../core/theme/app_typography.dart';
import '../../../../models.dart';

class QuestionBookScreen extends StatefulWidget {
  final List<InterviewQuestionDto> questions;

  const QuestionBookScreen({
    super.key,
    required this.questions,
  });

  @override
  State<QuestionBookScreen> createState() => _QuestionBookScreenState();
}

class _QuestionBookScreenState extends State<QuestionBookScreen> {
  String _selectedDifficulty = 'all';

  @override
  Widget build(BuildContext context) {
    final filtered = widget.questions.where((q) {
      if (_selectedDifficulty == 'all') return true;
      return q.difficulty.toLowerCase() == _selectedDifficulty;
    }).toList();

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          LayoutBuilder(
            builder: (context, constraints) {
              final isNarrow = constraints.maxWidth < 500;
              if (isNarrow) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Curated Interview Question Book', style: AppTypography.h1(context)),
                    const SizedBox(height: 4),
                    Text(
                      'High-leverage questions with interviewer intent, rubrics, and traps.',
                      style: AppTypography.body(context, color: AppColors.slate500),
                    ),
                    const SizedBox(height: 12),
                    DropdownButton<String>(
                      value: _selectedDifficulty,
                      underline: const SizedBox(),
                      items: const [
                        DropdownMenuItem(value: 'all', child: Text('All Difficulties')),
                        DropdownMenuItem(value: 'hard', child: Text('Hard')),
                        DropdownMenuItem(value: 'medium', child: Text('Medium')),
                        DropdownMenuItem(value: 'easy', child: Text('Easy')),
                      ],
                      onChanged: (val) {
                        if (val != null) setState(() => _selectedDifficulty = val);
                      },
                    ),
                  ],
                );
              }
              return Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Curated Interview Question Book', style: AppTypography.h1(context)),
                        const SizedBox(height: 4),
                        Text(
                          'High-leverage questions with interviewer intent, rubrics, and traps.',
                          style: AppTypography.body(context, color: AppColors.slate500),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 16),
                  DropdownButton<String>(
                    value: _selectedDifficulty,
                    underline: const SizedBox(),
                    items: const [
                      DropdownMenuItem(value: 'all', child: Text('All Difficulties')),
                      DropdownMenuItem(value: 'hard', child: Text('Hard')),
                      DropdownMenuItem(value: 'medium', child: Text('Medium')),
                      DropdownMenuItem(value: 'easy', child: Text('Easy')),
                    ],
                    onChanged: (val) {
                      if (val != null) setState(() => _selectedDifficulty = val);
                    },
                  ),
                ],
              );
            },
          ),
          const SizedBox(height: 20),

          ListView.separated(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: filtered.length,
            separatorBuilder: (_, __) => const SizedBox(height: 16),
            itemBuilder: (context, index) {
              final q = filtered[index];
              return _buildQuestionCard(context, q, index + 1);
            },
          ),
        ],
      ),
    );
  }

  Widget _buildQuestionCard(BuildContext context, InterviewQuestionDto q, int number) {
    Color badgeColor = AppColors.primary;
    if (q.difficulty.toLowerCase() == 'hard') {
      badgeColor = AppColors.rose;
    } else if (q.difficulty.toLowerCase() == 'medium') {
      badgeColor = AppColors.amber;
    }

    return Material(
      color: Theme.of(context).cardTheme.color,
      shape: RoundedRectangleBorder(
        side: BorderSide(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Padding(
        padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('QUESTION #$number', style: AppTypography.badge(color: AppColors.slate500)),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: badgeColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  q.difficulty.toUpperCase(),
                  style: AppTypography.badge(color: badgeColor),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Text(q.question, style: AppTypography.h2(context)),
          const SizedBox(height: 14),

          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(6),
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Icon(Icons.psychology_outlined, size: 18, color: AppColors.primary),
                const SizedBox(width: 8),
                Expanded(
                  child: RichText(
                    text: TextSpan(
                      children: [
                        const TextSpan(
                          text: 'Interviewer Intent: ',
                          style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12, color: AppColors.primary),
                        ),
                        TextSpan(
                          text: q.interviewerIntent,
                          style: TextStyle(fontSize: 12, color: Theme.of(context).colorScheme.onSurface),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 14),

          ExpansionTile(
            tilePadding: EdgeInsets.zero,
            title: Text('Answer Outline & Key Rubric Points', style: AppTypography.h3(context)),
            children: [
              Padding(
                padding: const EdgeInsets.only(top: 8, bottom: 8),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    ...q.answerOutline.map((point) {
                      return Padding(
                        padding: const EdgeInsets.only(bottom: 6),
                        child: Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            const Text('• ', style: TextStyle(fontWeight: FontWeight.bold)),
                            Expanded(child: Text(point, style: AppTypography.body(context))),
                          ],
                        ),
                      );
                    }),
                    if (q.strongAnswerRubric.isNotEmpty) ...[
                      const SizedBox(height: 10),
                      Text('Strong Answer Indicators:', style: AppTypography.h3(context, color: AppColors.emeraldDark)),
                      const SizedBox(height: 4),
                      ...q.strongAnswerRubric.map((r) => Padding(
                            padding: const EdgeInsets.only(bottom: 4),
                            child: Text('✔ $r', style: AppTypography.caption(context, color: AppColors.slate700)),
                          )),
                    ],
                    if (q.commonWeaknesses.isNotEmpty) ...[
                      const SizedBox(height: 10),
                      Text('Common Pitfalls:', style: AppTypography.h3(context, color: AppColors.roseDark)),
                      const SizedBox(height: 4),
                      ...q.commonWeaknesses.map((w) => Padding(
                            padding: const EdgeInsets.only(bottom: 4),
                            child: Text('✘ $w', style: AppTypography.caption(context, color: AppColors.slate700)),
                          )),
                    ],
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
    ),
  );
  }
}

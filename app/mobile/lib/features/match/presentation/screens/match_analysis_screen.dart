import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../core/theme/app_typography.dart';
import '../../../../models.dart';

class MatchAnalysisScreen extends StatelessWidget {
  final MatchAnalysisDto match;

  const MatchAnalysisScreen({
    super.key,
    required this.match,
  });

  @override
  Widget build(BuildContext context) {
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
                    Text('CV & JD Match Analysis', style: AppTypography.h1(context)),
                    const SizedBox(height: 4),
                    Text(
                      'Multi-dimensional alignment evaluated against job description requirements.',
                      style: AppTypography.body(context, color: AppColors.slate500),
                    ),
                    const SizedBox(height: 12),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                      decoration: BoxDecoration(
                        color: AppColors.primaryLight,
                        borderRadius: BorderRadius.circular(6),
                        border: Border.all(color: AppColors.primary.withValues(alpha: 0.3)),
                      ),
                      child: Text(
                        '${(match.overallMatchRatio * 100).round()}% Match',
                        style: AppTypography.mono(context, fontSize: 16, fontWeight: FontWeight.bold, color: AppColors.primary),
                      ),
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
                        Text('CV & JD Match Analysis', style: AppTypography.h1(context)),
                        const SizedBox(height: 4),
                        Text(
                          'Multi-dimensional alignment evaluated against job description requirements.',
                          style: AppTypography.body(context, color: AppColors.slate500),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 16),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                    decoration: BoxDecoration(
                      color: AppColors.primaryLight,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: AppColors.primary.withValues(alpha: 0.3)),
                    ),
                    child: Text(
                      '${(match.overallMatchRatio * 100).round()}% Match',
                      style: AppTypography.mono(context, fontSize: 16, fontWeight: FontWeight.bold, color: AppColors.primary),
                    ),
                  ),
                ],
              );
            },
          ),
          const SizedBox(height: 24),

          Row(
            children: [
              const Icon(Icons.check_circle_outline, color: AppColors.emerald, size: 20),
              const SizedBox(height: 4, width: 8),
              Text('Demonstrated Strengths & Verified Evidence', style: AppTypography.h2(context)),
            ],
          ),
          const SizedBox(height: 12),
          if (match.parsedStrengths.isEmpty)
            Text('No verified strengths mapped.', style: AppTypography.body(context, color: AppColors.slate500))
          else
            ListView.separated(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: match.parsedStrengths.length,
              separatorBuilder: (_, __) => const SizedBox(height: 10),
              itemBuilder: (context, index) {
                final s = match.parsedStrengths[index];
                return Container(
                  padding: const EdgeInsets.all(16),
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
                          Text(s.skill, style: AppTypography.h3(context)),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: AppColors.emeraldLight,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              s.relevance.toUpperCase(),
                              style: AppTypography.badge(color: AppColors.emeraldDark),
                            ),
                          ),
                        ],
                      ),
                      if (s.evidenceQuote.isNotEmpty) ...[
                        const SizedBox(height: 8),
                        Container(
                          padding: const EdgeInsets.all(10),
                          decoration: BoxDecoration(
                            color: Theme.of(context).colorScheme.surfaceContainerHighest,
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            '"${s.evidenceQuote}"',
                            style: AppTypography.caption(context, color: AppColors.slate600),
                          ),
                        ),
                      ],
                    ],
                  ),
                );
              },
            ),

          const SizedBox(height: 28),

          Row(
            children: [
              const Icon(Icons.warning_amber_rounded, color: AppColors.amber, size: 20),
              const SizedBox(height: 4, width: 8),
              Text('Identified Gaps & Missing Evidence', style: AppTypography.h2(context)),
            ],
          ),
          const SizedBox(height: 12),
          if (match.parsedGaps.isEmpty)
            Text('No gaps identified.', style: AppTypography.body(context, color: AppColors.slate500))
          else
            ListView.separated(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: match.parsedGaps.length,
              separatorBuilder: (_, __) => const SizedBox(height: 10),
              itemBuilder: (context, index) {
                final g = match.parsedGaps[index];
                return Container(
                  padding: const EdgeInsets.all(16),
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
                          Text(g.skill, style: AppTypography.h3(context)),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: AppColors.amberLight,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              g.importance.toUpperCase(),
                              style: AppTypography.badge(color: AppColors.amberDark),
                            ),
                          ),
                        ],
                      ),
                      if (g.requirementQuote.isNotEmpty) ...[
                        const SizedBox(height: 8),
                        Text('Role Requirement:', style: AppTypography.caption(context, color: AppColors.slate500)),
                        const SizedBox(height: 4),
                        Container(
                          padding: const EdgeInsets.all(10),
                          decoration: BoxDecoration(
                            color: Theme.of(context).colorScheme.surfaceContainerHighest,
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            '"${g.requirementQuote}"',
                            style: AppTypography.caption(context, color: AppColors.slate700),
                          ),
                        ),
                      ],
                      if (g.recommendedFocus.isNotEmpty) ...[
                        const SizedBox(height: 10),
                        Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            const Text('Recommendation: ', style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: AppColors.primary)),
                            Expanded(
                              child: Text(
                                g.recommendedFocus,
                                style: AppTypography.body(context, color: AppColors.slate600),
                              ),
                            ),
                          ],
                        ),
                      ],
                    ],
                  ),
                );
              },
            ),

          if (match.risks.isNotEmpty) ...[
            const SizedBox(height: 28),
            Row(
              children: [
                const Icon(Icons.error_outline, color: AppColors.rose, size: 20),
                const SizedBox(height: 4, width: 8),
                Text('Technical Risks', style: AppTypography.h2(context)),
              ],
            ),
            const SizedBox(height: 12),
            ...match.risks.map((risk) => Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: AppColors.roseLight,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: AppColors.rose.withValues(alpha: 0.3)),
                    ),
                    child: Text(
                      risk,
                      style: AppTypography.body(context, color: AppColors.roseDark),
                    ),
                  ),
                )),
          ],
        ],
      ),
    );
  }
}

import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../core/theme/app_typography.dart';
import '../../../../models.dart';

class CompanyDossierScreen extends StatelessWidget {
  final CompanyResearchDto company;

  const CompanyDossierScreen({
    super.key,
    required this.company,
  });

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(company.companyName, style: AppTypography.h1(context)),
          const SizedBox(height: 4),
          Text(
            'Verified company technical intelligence and citation provenance.',
            style: AppTypography.body(context, color: AppColors.slate500),
          ),
          const SizedBox(height: 24),

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
                Text('Executive Technical Summary', style: AppTypography.h2(context)),
                const SizedBox(height: 10),
                Text(
                  company.overview,
                  style: AppTypography.body(context, color: AppColors.slate700),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),

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
                Text('Verified Engineering Tech Stack', style: AppTypography.h2(context)),
                const SizedBox(height: 12),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: company.technologyStack.map((tech) {
                    return Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                      decoration: BoxDecoration(
                        color: Theme.of(context).colorScheme.surfaceContainerHighest,
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: Theme.of(context).dividerColor),
                      ),
                      child: Text(tech, style: AppTypography.mono(context, fontSize: 12)),
                    );
                  }).toList(),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),

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
                Text('Engineering Culture & Principles', style: AppTypography.h2(context)),
                const SizedBox(height: 12),
                ...company.engineeringCulturePillars.map((pillar) {
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Padding(
                          padding: EdgeInsets.only(top: 5, right: 8),
                          child: Icon(Icons.arrow_right, size: 16, color: AppColors.primary),
                        ),
                        Expanded(
                          child: Text(pillar, style: AppTypography.body(context)),
                        ),
                      ],
                    ),
                  );
                }),
              ],
            ),
          ),
          const SizedBox(height: 20),

          if (company.citations.isNotEmpty) ...[
            Text('Citation Sources & Provenance', style: AppTypography.h2(context)),
            const SizedBox(height: 12),
            ...company.citations.map((cite) {
              return Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: Theme.of(context).cardTheme.color,
                    border: Border.all(color: Theme.of(context).dividerColor),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(cite.title, style: AppTypography.h3(context)),
                      const SizedBox(height: 4),
                      Text(cite.sourceUrl, style: AppTypography.caption(context, color: AppColors.primary)),
                      if (cite.snippet.isNotEmpty) ...[
                        const SizedBox(height: 8),
                        Text(
                          '"${cite.snippet}"',
                          style: AppTypography.caption(context, color: AppColors.slate600),
                        ),
                      ],
                    ],
                  ),
                ),
              );
            }),
          ],
        ],
      ),
    );
  }
}

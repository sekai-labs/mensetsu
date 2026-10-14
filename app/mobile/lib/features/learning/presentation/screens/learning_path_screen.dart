import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../core/theme/app_typography.dart';
import '../../../../models.dart';

class LearningPathScreen extends StatefulWidget {
  final List<LearningUnitDto> lessons;
  final Future<void> Function(String lessonId) onToggleCompleted;

  const LearningPathScreen({
    super.key,
    required this.lessons,
    required this.onToggleCompleted,
  });

  @override
  State<LearningPathScreen> createState() => _LearningPathScreenState();
}

class _LearningPathScreenState extends State<LearningPathScreen> {
  String? _selectedLessonId;

  @override
  void initState() {
    super.initState();
    if (widget.lessons.isNotEmpty) {
      _selectedLessonId = widget.lessons.first.id;
    }
  }

  @override
  Widget build(BuildContext context) {
    final selectedLesson = widget.lessons.firstWhere(
      (l) => l.id == _selectedLessonId,
      orElse: () => widget.lessons.first,
    );

    return LayoutBuilder(
      builder: (context, constraints) {
        final isSplit = constraints.maxWidth > 750;

        if (isSplit) {
          return Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SizedBox(
                width: 320,
                child: _buildLessonList(context),
              ),
              const VerticalDivider(width: 1),
              Expanded(
                child: _buildLessonDetail(context, selectedLesson),
              ),
            ],
          );
        } else {
          return _buildLessonList(context, onSelectMobile: (lesson) {
            Navigator.of(context).push(
              MaterialPageRoute(
                builder: (_) => Scaffold(
                  appBar: AppBar(title: Text(lesson.title)),
                  body: _buildLessonDetail(context, lesson),
                ),
              ),
            );
          });
        }
      },
    );
  }

  Widget _buildLessonList(BuildContext context, {void Function(LearningUnitDto)? onSelectMobile}) {
    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: widget.lessons.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (context, index) {
        final lesson = widget.lessons[index];
        final isSelected = lesson.id == _selectedLessonId;

        return InkWell(
          onTap: () {
            if (onSelectMobile != null) {
              onSelectMobile(lesson);
            } else {
              setState(() => _selectedLessonId = lesson.id);
            }
          },
          borderRadius: BorderRadius.circular(8),
          child: Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: isSelected
                  ? Theme.of(context).colorScheme.surfaceContainerHighest
                  : Theme.of(context).cardTheme.color,
              border: Border.all(
                color: isSelected ? AppColors.primary : Theme.of(context).dividerColor,
                width: isSelected ? 1.5 : 1.0,
              ),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              children: [
                Checkbox(
                  value: lesson.completed,
                  activeColor: AppColors.emerald,
                  onChanged: (_) => widget.onToggleCompleted(lesson.id),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        lesson.skill.toUpperCase(),
                        style: AppTypography.badge(color: AppColors.primary),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        lesson.title,
                        style: AppTypography.h3(context),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        '${lesson.estimatedMinutes} mins • ${lesson.chapters.length} chapters',
                        style: AppTypography.caption(context),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildLessonDetail(BuildContext context, LearningUnitDto lesson) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(lesson.skill.toUpperCase(), style: AppTypography.badge(color: AppColors.primary)),
                    const SizedBox(height: 6),
                    Text(lesson.title, style: AppTypography.h1(context)),
                  ],
                ),
              ),
              IconButton(
                icon: Icon(
                  lesson.completed ? Icons.check_circle : Icons.check_circle_outline,
                  color: lesson.completed ? AppColors.emerald : AppColors.slate400,
                  size: 28,
                ),
                tooltip: lesson.completed ? 'Mark incomplete' : 'Mark completed',
                onPressed: () => widget.onToggleCompleted(lesson.id),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Text(lesson.summary, style: AppTypography.body(context, color: AppColors.slate600)),
          const SizedBox(height: 20),

          if (lesson.chapters.isNotEmpty) ...[
            Text('Lesson Timeline & Chapters', style: AppTypography.h2(context)),
            const SizedBox(height: 10),
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Theme.of(context).cardTheme.color,
                border: Border.all(color: Theme.of(context).dividerColor),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                children: lesson.chapters.map((ch) {
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Theme.of(context).colorScheme.surfaceContainerHighest,
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(ch.timestamp, style: AppTypography.mono(context, fontSize: 11)),
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(ch.title, style: AppTypography.h3(context)),
                              if (ch.summary != null)
                                Text(ch.summary!, style: AppTypography.caption(context)),
                            ],
                          ),
                        ),
                      ],
                    ),
                  );
                }).toList(),
              ),
            ),
            const SizedBox(height: 20),
          ],

          Text('Key Invariants & Lesson Material', style: AppTypography.h2(context)),
          const SizedBox(height: 10),
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).cardTheme.color,
              border: Border.all(color: Theme.of(context).dividerColor),
              borderRadius: BorderRadius.circular(8),
            ),
            child: SelectableText(
              lesson.contentMarkdown,
              style: AppTypography.body(context),
            ),
          ),
        ],
      ),
    );
  }
}

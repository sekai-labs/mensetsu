import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../models.dart';
import '../../../../api_client.dart';
import '../../../match/presentation/screens/match_analysis_screen.dart';
import '../../../company/presentation/screens/company_dossier_screen.dart';
import '../../../learning/presentation/screens/learning_path_screen.dart';
import '../../../questions/presentation/screens/question_book_screen.dart';
import '../../../settings/presentation/screens/llm_settings_screen.dart';
import 'workspace_overview_screen.dart';

class WorkspaceDetailShell extends StatefulWidget {
  final String workspaceId;
  final MensetsuApiClient api;

  const WorkspaceDetailShell({
    super.key,
    required this.workspaceId,
    required this.api,
  });

  @override
  State<WorkspaceDetailShell> createState() => _WorkspaceDetailShellState();
}

class _WorkspaceDetailShellState extends State<WorkspaceDetailShell> {
  int _currentIndex = 0;
  bool _isLoading = true;
  WorkspaceDto? _workspace;
  MatchAnalysisDto? _match;
  CompanyResearchDto? _company;
  List<InterviewQuestionDto> _questions = [];
  List<LearningUnitDto> _lessons = [];

  @override
  void initState() {
    super.initState();
    _loadAllData();
  }

  Future<void> _loadAllData() async {
    setState(() => _isLoading = true);
    final ws = await widget.api.getWorkspace(widget.workspaceId);
    final match = await widget.api.getWorkspaceMatch(widget.workspaceId);
    final comp = await widget.api.getWorkspaceCompany(widget.workspaceId);
    final questions = await widget.api.getWorkspaceQuestions(widget.workspaceId);
    final lessons = await widget.api.getWorkspaceLearning(widget.workspaceId);

    if (mounted) {
      setState(() {
        _workspace = ws;
        _match = match;
        _company = comp;
        _questions = questions;
        _lessons = lessons;
        _isLoading = false;
      });
    }
  }

  Future<void> _toggleLessonCompleted(String lessonId) async {
    await widget.api.toggleLessonCompleted(widget.workspaceId, lessonId);
    setState(() {
      _lessons = _lessons.map((l) {
        if (l.id == lessonId) {
          return l.copyWith(completed: !l.completed);
        }
        return l;
      }).toList();
    });
  }

  Widget _buildContent() {
    if (_workspace == null) {
      return const Center(child: Text('Workspace not found'));
    }

    switch (_currentIndex) {
      case 0:
        return WorkspaceOverviewScreen(
          workspace: _workspace!,
          match: _match,
          company: _company,
          questions: _questions,
          lessons: _lessons,
          onNavigateToMatch: () => setState(() => _currentIndex = 1),
          onNavigateToCompany: () => setState(() => _currentIndex = 2),
          onNavigateToLearning: () => setState(() => _currentIndex = 3),
          onNavigateToQuestions: () => setState(() => _currentIndex = 4),
        );
      case 1:
        return _match != null
            ? MatchAnalysisScreen(match: _match!)
            : const Center(child: Text('No match analysis data'));
      case 2:
        return _company != null
            ? CompanyDossierScreen(company: _company!)
            : const Center(child: Text('No company research data'));
      case 3:
        return LearningPathScreen(
          lessons: _lessons,
          onToggleCompleted: _toggleLessonCompleted,
        );
      case 4:
        return QuestionBookScreen(questions: _questions);
      case 5:
        return LlmSettingsScreen(api: widget.api);
      default:
        return const SizedBox();
    }
  }

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.sizeOf(context).width;
    final isMobile = width < 600;
    final isTablet = width >= 600 && width <= 1024;
    final isDesktop = width > 1024;

    final title = _workspace?.companyName ?? 'Workspace';

    if (_isLoading) {
      return Scaffold(
        appBar: AppBar(title: Text(title)),
        body: const Center(child: CircularProgressIndicator()),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: AppColors.primary,
                borderRadius: BorderRadius.circular(4),
              ),
              child: const Text('面', style: TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold)),
            ),
            const SizedBox(width: 10),
            Flexible(
              child: Text(
                '$title (${_workspace?.roleTitle ?? ""})',
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600),
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Reload workspace data',
            onPressed: _loadAllData,
          ),
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            tooltip: 'LLM Settings',
            onPressed: () => setState(() => _currentIndex = 5),
          ),
          const SizedBox(width: 8),
        ],
      ),
      bottomNavigationBar: isMobile
          ? BottomNavigationBar(
              currentIndex: _currentIndex.clamp(0, 4),
              onTap: (index) => setState(() => _currentIndex = index),
              items: const [
                BottomNavigationBarItem(icon: Icon(Icons.dashboard_outlined), activeIcon: Icon(Icons.dashboard), label: 'Overview'),
                BottomNavigationBarItem(icon: Icon(Icons.compare_arrows_outlined), activeIcon: Icon(Icons.compare_arrows), label: 'Match'),
                BottomNavigationBarItem(icon: Icon(Icons.business_outlined), activeIcon: Icon(Icons.business), label: 'Company'),
                BottomNavigationBarItem(icon: Icon(Icons.school_outlined), activeIcon: Icon(Icons.school), label: 'Learning'),
                BottomNavigationBarItem(icon: Icon(Icons.quiz_outlined), activeIcon: Icon(Icons.quiz), label: 'Questions'),
              ],
            )
          : null,
      body: Row(
        children: [
          if (isTablet)
            NavigationRail(
              selectedIndex: _currentIndex.clamp(0, 5),
              onDestinationSelected: (idx) => setState(() => _currentIndex = idx),
              labelType: NavigationRailLabelType.all,
              destinations: const [
                NavigationRailDestination(icon: Icon(Icons.dashboard_outlined), selectedIcon: Icon(Icons.dashboard), label: Text('Overview')),
                NavigationRailDestination(icon: Icon(Icons.compare_arrows_outlined), selectedIcon: Icon(Icons.compare_arrows), label: Text('Match')),
                NavigationRailDestination(icon: Icon(Icons.business_outlined), selectedIcon: Icon(Icons.business), label: Text('Company')),
                NavigationRailDestination(icon: Icon(Icons.school_outlined), selectedIcon: Icon(Icons.school), label: Text('Learning')),
                NavigationRailDestination(icon: Icon(Icons.quiz_outlined), selectedIcon: Icon(Icons.quiz), label: Text('Questions')),
                NavigationRailDestination(icon: Icon(Icons.settings_outlined), selectedIcon: Icon(Icons.settings), label: Text('Settings')),
              ],
            ),

          if (isDesktop)
            Container(
              width: 250,
              decoration: BoxDecoration(
                color: Theme.of(context).cardTheme.color,
                border: Border(right: BorderSide(color: Theme.of(context).dividerColor)),
              ),
              child: ListView(
                padding: const EdgeInsets.symmetric(vertical: 12),
                children: [
                  _sidebarItem(0, Icons.dashboard_outlined, Icons.dashboard, 'Workspace Overview'),
                  _sidebarItem(1, Icons.compare_arrows_outlined, Icons.compare_arrows, 'Match Analysis'),
                  _sidebarItem(2, Icons.business_outlined, Icons.business, 'Company Dossier'),
                  _sidebarItem(3, Icons.school_outlined, Icons.school, 'Learning Path'),
                  _sidebarItem(4, Icons.quiz_outlined, Icons.quiz, 'Question Book'),
                  const Divider(height: 24),
                  _sidebarItem(5, Icons.settings_outlined, Icons.settings, 'LLM Settings'),
                ],
              ),
            ),

          Expanded(
            child: _buildContent(),
          ),
        ],
      ),
    );
  }

  Widget _sidebarItem(int index, IconData icon, IconData activeIcon, String label) {
    final isSelected = _currentIndex == index;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 3),
      child: InkWell(
        onTap: () => setState(() => _currentIndex = index),
        borderRadius: BorderRadius.circular(6),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: isSelected ? AppColors.primaryLight : Colors.transparent,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Row(
            children: [
              Icon(
                isSelected ? activeIcon : icon,
                size: 18,
                color: isSelected ? AppColors.primary : AppColors.slate500,
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  label,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: isSelected ? FontWeight.w600 : FontWeight.w500,
                    color: isSelected ? AppColors.primary : AppColors.slate700,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

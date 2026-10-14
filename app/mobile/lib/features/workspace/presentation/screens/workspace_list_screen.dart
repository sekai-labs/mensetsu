import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../core/theme/app_typography.dart';
import '../../../../models.dart';
import '../../../../api_client.dart';
import 'workspace_detail_shell.dart';
import '../../../settings/presentation/screens/llm_settings_screen.dart';

class WorkspaceListScreen extends StatefulWidget {
  final MensetsuApiClient api;

  const WorkspaceListScreen({
    super.key,
    required this.api,
  });

  @override
  State<WorkspaceListScreen> createState() => _WorkspaceListScreenState();
}

class _WorkspaceListScreenState extends State<WorkspaceListScreen> {
  late Future<List<WorkspaceDto>> _workspacesFuture;
  String _selectedFilter = 'all';

  @override
  void initState() {
    super.initState();
    _refreshWorkspaces();
  }

  void _refreshWorkspaces() {
    setState(() {
      _workspacesFuture = widget.api.listWorkspaces().catchError((_) => <WorkspaceDto>[]);
    });
  }

  void _openNewWorkspaceDialog() {
    final companyController = TextEditingController();
    final roleController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('New Preparation Workspace'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Create an evidence-backed interview research workspace.',
                style: AppTypography.caption(context),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: companyController,
                decoration: const InputDecoration(
                  labelText: 'Target Company',
                  hintText: 'e.g. ScaleVector, Acme Systems',
                  border: OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: roleController,
                decoration: const InputDecoration(
                  labelText: 'Role Title',
                  hintText: 'e.g. Staff Systems Architect',
                  border: OutlineInputBorder(),
                ),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            ElevatedButton(
              onPressed: () async {
                final company = companyController.text.trim();
                final role = roleController.text.trim();
                if (company.isNotEmpty && role.isNotEmpty) {
                  final newWs = await widget.api.createWorkspace(
                    companyName: company,
                    roleTitle: role,
                  );
                  if (context.mounted) {
                    Navigator.of(context).pop();
                    _refreshWorkspaces();
                    Navigator.of(context).push(
                      MaterialPageRoute(
                        builder: (_) => WorkspaceDetailShell(
                          workspaceId: newWs.id,
                          api: widget.api,
                        ),
                      ),
                    );
                  }
                }
              },
              child: const Text('Create Workspace'),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.sizeOf(context).width;
    final isDesktop = width > 900;

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
            const Text('Mensetsu Workspaces', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 16)),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            tooltip: 'Settings',
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => Scaffold(
                    appBar: AppBar(title: const Text('Settings')),
                    body: LlmSettingsScreen(api: widget.api),
                  ),
                ),
              );
            },
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh',
            onPressed: _refreshWorkspaces,
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
            child: LayoutBuilder(
              builder: (context, constraints) {
                final isNarrow = constraints.maxWidth < 650;
                if (isNarrow) {
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      SingleChildScrollView(
                        scrollDirection: Axis.horizontal,
                        child: Row(
                          children: [
                            _filterChip('all', 'All Workspaces'),
                            const SizedBox(width: 8),
                            _filterChip('ready', 'Ready'),
                            const SizedBox(width: 8),
                            _filterChip('processing', 'Processing'),
                          ],
                        ),
                      ),
                      const SizedBox(height: 12),
                      ElevatedButton.icon(
                        onPressed: _openNewWorkspaceDialog,
                        icon: const Icon(Icons.add, size: 16),
                        label: const Text('New Workspace'),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: AppColors.primary,
                          foregroundColor: Colors.white,
                        ),
                      ),
                    ],
                  );
                }
                return Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Expanded(
                      child: SingleChildScrollView(
                        scrollDirection: Axis.horizontal,
                        child: Row(
                          children: [
                            _filterChip('all', 'All Workspaces'),
                            const SizedBox(width: 8),
                            _filterChip('ready', 'Ready'),
                            const SizedBox(width: 8),
                            _filterChip('processing', 'Processing'),
                          ],
                        ),
                      ),
                    ),
                    const SizedBox(width: 12),
                    ElevatedButton.icon(
                      onPressed: _openNewWorkspaceDialog,
                      icon: const Icon(Icons.add, size: 16),
                      label: const Text('New Workspace'),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppColors.primary,
                        foregroundColor: Colors.white,
                      ),
                    ),
                  ],
                );
              },
            ),
          ),

          Expanded(
            child: FutureBuilder<List<WorkspaceDto>>(
              future: _workspacesFuture,
              builder: (context, snapshot) {
                if (snapshot.connectionState == ConnectionState.waiting) {
                  return const Center(child: CircularProgressIndicator());
                }

                final allWorkspaces = snapshot.data ?? [];
                final workspaces = allWorkspaces.where((ws) {
                  if (_selectedFilter == 'all') return true;
                  return ws.status.toLowerCase() == _selectedFilter;
                }).toList();

                if (workspaces.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(Icons.work_outline, size: 48, color: AppColors.slate400),
                        const SizedBox(height: 14),
                        Text(
                          'No active workspaces found',
                          style: AppTypography.h2(context),
                        ),
                        const SizedBox(height: 6),
                        Text(
                          'Initialize a target company workspace to generate evidence rubrics.',
                          style: AppTypography.body(context, color: AppColors.slate500),
                        ),
                        const SizedBox(height: 18),
                        ElevatedButton(
                          onPressed: _openNewWorkspaceDialog,
                          child: const Text('Create New Workspace'),
                        ),
                      ],
                    ),
                  );
                }

                return GridView.builder(
                  padding: const EdgeInsets.all(24),
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: isDesktop ? 3 : (width > 600 ? 2 : 1),
                    mainAxisSpacing: 16,
                    crossAxisSpacing: 16,
                    childAspectRatio: isDesktop ? 1.8 : 2.2,
                  ),
                  itemCount: workspaces.length,
                  itemBuilder: (context, index) {
                    final ws = workspaces[index];
                    return _buildWorkspaceCard(context, ws);
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _filterChip(String key, String label) {
    final isSelected = _selectedFilter == key;
    return ChoiceChip(
      label: Text(label),
      selected: isSelected,
      onSelected: (selected) {
        if (selected) setState(() => _selectedFilter = key);
      },
    );
  }

  Widget _buildWorkspaceCard(BuildContext context, WorkspaceDto ws) {
    return InkWell(
      onTap: () {
        Navigator.of(context).push(
          MaterialPageRoute(
            builder: (_) => WorkspaceDetailShell(
              workspaceId: ws.id,
              api: widget.api,
            ),
          ),
        );
      },
      borderRadius: BorderRadius.circular(8),
      child: Container(
        padding: const EdgeInsets.all(18),
        decoration: BoxDecoration(
          color: Theme.of(context).cardTheme.color,
          border: Border.all(color: Theme.of(context).dividerColor),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
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
                    ws.status.toUpperCase(),
                    style: AppTypography.badge(color: AppColors.emeraldDark),
                  ),
                ),
                Text(
                  '${ws.progressPercent}% Ready',
                  style: AppTypography.mono(context, fontSize: 11, color: AppColors.slate500),
                ),
              ],
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  ws.companyName,
                  style: AppTypography.h2(context),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 4),
                Text(
                  ws.roleTitle,
                  style: AppTypography.body(context, color: AppColors.slate500),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Open Detailed Analysis →',
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                    color: AppColors.primary,
                  ),
                ),
                const Icon(Icons.arrow_forward, size: 14, color: AppColors.primary),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

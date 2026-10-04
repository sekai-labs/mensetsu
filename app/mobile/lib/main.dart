import 'package:flutter/material.dart';
import 'models.dart';
import 'api_client.dart';

void main() {
  runApp(const MensetsuApp());
}

class MensetsuApp extends StatelessWidget {
  const MensetsuApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Mensetsu',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF4F46E5),
          brightness: Brightness.light,
        ),
        fontFamily: 'Inter',
      ),
      darkTheme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF4F46E5),
          brightness: Brightness.dark,
        ),
      ),
      home: const WorkspaceListScreen(),
    );
  }
}

class WorkspaceListScreen extends StatefulWidget {
  const WorkspaceListScreen({super.key});

  @override
  State<WorkspaceListScreen> createState() => _WorkspaceListScreenState();
}

class _WorkspaceListScreenState extends State<WorkspaceListScreen> {
  final MensetsuApiClient _api = MensetsuApiClient();
  late Future<List<WorkspaceDto>> _workspacesFuture;

  @override
  void initState() {
    super.initState();
    _workspacesFuture = _api.listWorkspaces().catchError((_) => <WorkspaceDto>[]);
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = MediaQuery.sizeOf(context).width > 800;

    return Scaffold(
      appBar: AppBar(
        title: const Row(
          children: [
            Text('面 ', style: TextStyle(color: Color(0xFF4F46E5), fontWeight: FontWeight.bold)),
            Text('Mensetsu Workspace', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 18)),
          ],
        ),
      ),
      body: FutureBuilder<List<WorkspaceDto>>(
        future: _workspacesFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return const Center(child: CircularProgressIndicator());
          }

          final workspaces = snapshot.data ?? [];
          if (workspaces.isEmpty) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.work_outline, size: 48, color: Colors.grey),
                  const SizedBox(height: 12),
                  const Text('No interview workspaces active', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500)),
                  const SizedBox(height: 4),
                  const Text('Create a preparation workspace using the web wizard.', style: TextStyle(color: Colors.grey, fontSize: 13)),
                  const SizedBox(height: 16),
                  ElevatedButton(
                    onPressed: () => setState(() {
                      _workspacesFuture = _api.listWorkspaces().catchError((_) => <WorkspaceDto>[]);
                    }),
                    child: const Text('Refresh'),
                  )
                ],
              ),
            );
          }

          return GridView.builder(
            padding: const EdgeInsets.all(16),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: isDesktop ? 3 : 1,
              mainAxisSpacing: 16,
              crossAxisSpacing: 16,
              childAspectRatio: isDesktop ? 2.0 : 2.5,
            ),
            itemCount: workspaces.length,
            itemBuilder: (context, index) {
              final ws = workspaces[index];
              return Card(
                elevation: 0,
                shape: RoundedRectangleBorder(
                  side: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                            decoration: BoxDecoration(
                              color: const Color(0xFF10B981).withValues(alpha: 0.1),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              ws.status.toUpperCase(),
                              style: const TextStyle(color: Color(0xFF10B981), fontSize: 10, fontWeight: FontWeight.bold),
                            ),
                          ),
                          Text('${ws.progressPercent}%', style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                        ],
                      ),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(ws.companyName, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                          Text(ws.roleTitle, style: const TextStyle(fontSize: 13, color: Colors.grey)),
                        ],
                      ),
                      const Text('View Detailed Analysis →', style: TextStyle(fontSize: 12, color: Color(0xFF4F46E5), fontWeight: FontWeight.w600)),
                    ],
                  ),
                ),
              );
            },
          );
        },
      ),
    );
  }
}

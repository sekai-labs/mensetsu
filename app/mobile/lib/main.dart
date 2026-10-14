import 'package:flutter/material.dart';
import 'core/theme/app_theme.dart';
import 'api_client.dart';
import 'features/workspace/presentation/screens/workspace_list_screen.dart';

void main() {
  runApp(const MensetsuApp());
}

class MensetsuApp extends StatelessWidget {
  const MensetsuApp({super.key});

  @override
  Widget build(BuildContext context) {
    final api = MensetsuApiClient();

    return MaterialApp(
      title: 'Mensetsu',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.lightTheme,
      darkTheme: AppTheme.darkTheme,
      themeMode: ThemeMode.system,
      home: WorkspaceListScreen(api: api),
    );
  }
}

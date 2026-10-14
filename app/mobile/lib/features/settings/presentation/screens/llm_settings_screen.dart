import 'package:flutter/material.dart';
import '../../../../core/theme/app_colors.dart';
import '../../../../core/theme/app_typography.dart';
import '../../../../models.dart';
import '../../../../api_client.dart';

class LlmSettingsScreen extends StatefulWidget {
  final MensetsuApiClient api;

  const LlmSettingsScreen({
    super.key,
    required this.api,
  });

  @override
  State<LlmSettingsScreen> createState() => _LlmSettingsScreenState();
}

class _LlmSettingsScreenState extends State<LlmSettingsScreen> {
  late Future<List<LlmConfigDto>> _settingsFuture;

  @override
  void initState() {
    super.initState();
    _settingsFuture = widget.api.getLlmSettings();
  }

  void _showEditKeyDialog(LlmConfigDto config) {
    final keyController = TextEditingController();
    showDialog(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: Text('Configure ${config.provider.toUpperCase()} Key'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Provide custom API key for this provider.', style: AppTypography.caption(context)),
              const SizedBox(height: 12),
              TextField(
                controller: keyController,
                obscureText: true,
                decoration: const InputDecoration(
                  labelText: 'API Key',
                  border: OutlineInputBorder(),
                  hintText: 'sk-...',
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
                final newKey = keyController.text.trim();
                if (newKey.isNotEmpty) {
                  await widget.api.updateLlmSetting({
                    'provider': config.provider,
                    'api_key': newKey,
                    'is_shared': config.isShared,
                  });
                  setState(() {
                    _settingsFuture = widget.api.getLlmSettings();
                  });
                }
                if (context.mounted) Navigator.of(context).pop();
              },
              child: const Text('Save'),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('LLM Provider Configuration', style: AppTypography.h1(context)),
          const SizedBox(height: 4),
          Text(
            'Configure underlying LLMs for research synthesis and interview questions.',
            style: AppTypography.body(context, color: AppColors.slate500),
          ),
          const SizedBox(height: 24),

          FutureBuilder<List<LlmConfigDto>>(
            future: _settingsFuture,
            builder: (context, snapshot) {
              if (snapshot.connectionState == ConnectionState.waiting) {
                return const Center(child: Padding(padding: EdgeInsets.all(32), child: CircularProgressIndicator()));
              }

              final configs = snapshot.data ?? [];
              return ListView.separated(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: configs.length,
                separatorBuilder: (_, __) => const SizedBox(height: 12),
                itemBuilder: (context, index) {
                  final config = configs[index];
                  return Container(
                    padding: const EdgeInsets.all(16),
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
                              Row(
                                children: [
                                  Text(config.provider.toUpperCase(), style: AppTypography.h3(context)),
                                  const SizedBox(width: 8),
                                  if (config.hasKey)
                                    Container(
                                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                      decoration: BoxDecoration(
                                        color: AppColors.emeraldLight,
                                        borderRadius: BorderRadius.circular(4),
                                      ),
                                      child: Text('KEY SET', style: AppTypography.badge(color: AppColors.emeraldDark)),
                                    )
                                  else
                                    Container(
                                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                      decoration: BoxDecoration(
                                        color: AppColors.slate100,
                                        borderRadius: BorderRadius.circular(4),
                                      ),
                                      child: Text('NO KEY', style: AppTypography.badge(color: AppColors.slate500)),
                                    ),
                                ],
                              ),
                              const SizedBox(height: 6),
                              Text(
                                'Chat Model: ${config.modelChat ?? "Default"}',
                                style: AppTypography.caption(context),
                              ),
                              if (config.maskedKey != null) ...[
                                const SizedBox(height: 2),
                                Text(
                                  'Key: ${config.maskedKey}',
                                  style: AppTypography.mono(context, fontSize: 11, color: AppColors.slate500),
                                ),
                              ],
                            ],
                          ),
                        ),
                        OutlinedButton(
                          onPressed: () => _showEditKeyDialog(config),
                          child: const Text('Configure'),
                        ),
                      ],
                    ),
                  );
                },
              );
            },
          ),
        ],
      ),
    );
  }
}

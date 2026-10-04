import 'package:flutter_test/flutter_test.dart';
import 'package:mensetsu_mobile/models.dart';

void main() {
  group('Flutter Client Model Deserialization', () {
    test('WorkspaceDto parses from JSON', () {
      final json = {
        'id': 'ws_123',
        'company_name': 'ScaleVector',
        'role_title': 'Staff Systems Architect',
        'status': 'ready',
        'progress_percent': 100,
      };

      final dto = WorkspaceDto.fromJson(json);
      expect(dto.id, 'ws_123');
      expect(dto.companyName, 'ScaleVector');
      expect(dto.roleTitle, 'Staff Systems Architect');
      expect(dto.status, 'ready');
      expect(dto.progressPercent, 100);
    });

    test('MatchAnalysisDto parses strengths and gaps', () {
      final json = {
        'workspace_id': 'ws_123',
        'overall_match_ratio': 0.9,
        'strengths': [
          {'skill': 'Rust', 'evidence_quote': 'Core maintainer', 'relevance': 'High'}
        ],
        'gaps': [
          {'skill': 'eBPF', 'requirement_quote': 'Observability', 'importance': 'preferred'}
        ],
      };

      final dto = MatchAnalysisDto.fromJson(json);
      expect(dto.workspaceId, 'ws_123');
      expect(dto.overallMatchRatio, 0.9);
      expect(dto.strengths.length, 1);
      expect(dto.gaps.length, 1);
    });

    test('UserProfileDto parses role from JSON', () {
      final json = {
        'id': 'u1',
        'email': 'admin@company.com',
        'name': 'Admin User',
        'role': 'admin',
      };
      final user = UserProfileDto.fromJson(json);
      expect(user.role, 'admin');
      expect(user.name, 'Admin User');
    });

    test('LearningUnitDto parses video and chapter marks', () {
      final json = {
        'id': 'lu_1',
        'skill': 'Distributed Systems',
        'title': 'Mastering Raft',
        'estimated_minutes': 30,
        'summary': 'Leader election in Raft',
        'content_markdown': '### Invariants',
        'video_url': 'https://www.youtube-nocookie.com/embed/t9fT7n8k_uM',
        'video_title': 'Raft Consensus',
        'chapters': [
          {'timestamp': '00:00', 'title': 'Intro', 'summary': 'Overview'},
          {'timestamp': '05:00', 'title': 'Heartbeats', 'summary': 'Timeout'}
        ],
        'completed': true,
      };
      final unit = LearningUnitDto.fromJson(json);
      expect(unit.completed, true);
      expect(unit.videoUrl, isNotNull);
      expect(unit.chapters.length, 2);
      expect(unit.chapters[1].title, 'Heartbeats');
    });

    test('LlmConfigDto parses sharing and masked keys', () {
      final json = {
        'provider': 'openai',
        'is_shared': true,
        'has_key': true,
        'masked_key': 'sk-...1234',
        'is_inherited': false,
      };
      final config = LlmConfigDto.fromJson(json);
      expect(config.provider, 'openai');
      expect(config.isShared, true);
      expect(config.hasKey, true);
      expect(config.maskedKey, 'sk-...1234');
    });
  });
}

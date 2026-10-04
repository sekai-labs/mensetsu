import 'dart:convert';
import 'package:http/http.dart' as http;
import 'models.dart';

class MensetsuApiClient {
  final String baseUrl;
  final http.Client client;

  MensetsuApiClient({this.baseUrl = 'http://127.0.0.1:3000', http.Client? client})
      : client = client ?? http.Client();

  Future<List<WorkspaceDto>> listWorkspaces() async {
    final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces'));
    if (response.statusCode == 200) {
      final List<dynamic> list = jsonDecode(response.body);
      return list.map((item) => WorkspaceDto.fromJson(item as Map<String, dynamic>)).toList();
    }
    throw Exception('Failed to load workspaces');
  }

  Future<WorkspaceDto> getWorkspace(String id) async {
    final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$id'));
    if (response.statusCode == 200) {
      return WorkspaceDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
    }
    throw Exception('Failed to load workspace $id');
  }

  Future<MatchAnalysisDto> getWorkspaceMatch(String id) async {
    final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$id/match'));
    if (response.statusCode == 200) {
      return MatchAnalysisDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
    }
    throw Exception('Failed to load match analysis for $id');
  }

  Future<List<InterviewQuestionDto>> getWorkspaceQuestions(String id) async {
    final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$id/questions'));
    if (response.statusCode == 200) {
      final List<dynamic> list = jsonDecode(response.body);
      return list.map((item) => InterviewQuestionDto.fromJson(item as Map<String, dynamic>)).toList();
    }
    throw Exception('Failed to load questions for $id');
  }

  Future<UserProfileDto> getProfile({String? token}) async {
    final headers = <String, String>{};
    if (token != null) {
      headers['Authorization'] = 'Bearer $token';
    }
    final response = await client.get(
      Uri.parse('$baseUrl/api/v1/settings/profile'),
      headers: headers,
    );
    if (response.statusCode == 200) {
      return UserProfileDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
    }
    throw Exception('Failed to load profile');
  }

  Future<List<LlmConfigDto>> getLlmSettings({String? token}) async {
    final headers = <String, String>{};
    if (token != null) {
      headers['Authorization'] = 'Bearer $token';
    }
    final response = await client.get(
      Uri.parse('$baseUrl/api/v1/settings/llm'),
      headers: headers,
    );
    if (response.statusCode == 200) {
      final List<dynamic> list = jsonDecode(response.body);
      return list.map((item) => LlmConfigDto.fromJson(item as Map<String, dynamic>)).toList();
    }
    throw Exception('Failed to load LLM settings');
  }

  Future<LlmConfigDto> updateLlmSetting(Map<String, dynamic> payload, {String? token}) async {
    final headers = <String, String>{'Content-Type': 'application/json'};
    if (token != null) {
      headers['Authorization'] = 'Bearer $token';
    }
    final response = await client.put(
      Uri.parse('$baseUrl/api/v1/settings/llm'),
      headers: headers,
      body: jsonEncode(payload),
    );
    if (response.statusCode == 200) {
      return LlmConfigDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
    }
    throw Exception('Failed to update LLM setting');
  }

  Future<bool> toggleLessonCompleted(String workspaceId, String lessonId, {String? token}) async {
    final headers = <String, String>{};
    if (token != null) {
      headers['Authorization'] = 'Bearer $token';
    }
    final response = await client.put(
      Uri.parse('$baseUrl/api/v1/workspaces/$workspaceId/learning/$lessonId/toggle'),
      headers: headers,
    );
    if (response.statusCode == 200) {
      final Map<String, dynamic> body = jsonDecode(response.body);
      return body['completed'] as bool? ?? false;
    }
    throw Exception('Failed to toggle lesson completed');
  }
}

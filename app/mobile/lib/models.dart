class WorkspaceDto {
  final String id;
  final String companyName;
  final String roleTitle;
  final String status;
  final int progressPercent;

  WorkspaceDto({
    required this.id,
    required this.companyName,
    required this.roleTitle,
    required this.status,
    required this.progressPercent,
  });

  factory WorkspaceDto.fromJson(Map<String, dynamic> json) {
    return WorkspaceDto(
      id: json['id'] as String? ?? '',
      companyName: json['company_name'] as String? ?? '',
      roleTitle: json['role_title'] as String? ?? '',
      status: json['status'] as String? ?? 'pending',
      progressPercent: (json['progress_percent'] as num?)?.toInt() ?? 0,
    );
  }
}

class MatchAnalysisDto {
  final String workspaceId;
  final double overallMatchRatio;
  final List<dynamic> strengths;
  final List<dynamic> gaps;

  MatchAnalysisDto({
    required this.workspaceId,
    required this.overallMatchRatio,
    required this.strengths,
    required this.gaps,
  });

  factory MatchAnalysisDto.fromJson(Map<String, dynamic> json) {
    return MatchAnalysisDto(
      workspaceId: json['workspace_id'] as String? ?? '',
      overallMatchRatio: (json['overall_match_ratio'] as num?)?.toDouble() ?? 0.0,
      strengths: json['strengths'] as List<dynamic>? ?? [],
      gaps: json['gaps'] as List<dynamic>? ?? [],
    );
  }
}

class InterviewQuestionDto {
  final String id;
  final String question;
  final String difficulty;
  final String interviewerIntent;
  final List<String> answerOutline;

  InterviewQuestionDto({
    required this.id,
    required this.question,
    required this.difficulty,
    required this.interviewerIntent,
    required this.answerOutline,
  });

  factory InterviewQuestionDto.fromJson(Map<String, dynamic> json) {
    return InterviewQuestionDto(
      id: json['id'] as String? ?? '',
      question: json['question'] as String? ?? '',
      difficulty: json['difficulty'] as String? ?? 'medium',
      interviewerIntent: json['interviewer_intent'] as String? ?? '',
      answerOutline: (json['answer_outline'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
    );
  }
}

class UserProfileDto {
  final String id;
  final String email;
  final String name;
  final String role;

  UserProfileDto({
    required this.id,
    required this.email,
    required this.name,
    required this.role,
  });

  factory UserProfileDto.fromJson(Map<String, dynamic> json) {
    return UserProfileDto(
      id: json['id'] as String? ?? '',
      email: json['email'] as String? ?? '',
      name: json['name'] as String? ?? '',
      role: json['role'] as String? ?? 'member',
    );
  }
}

class LessonChapterDto {
  final String timestamp;
  final String title;
  final String? summary;

  LessonChapterDto({
    required this.timestamp,
    required this.title,
    this.summary,
  });

  factory LessonChapterDto.fromJson(Map<String, dynamic> json) {
    return LessonChapterDto(
      timestamp: json['timestamp'] as String? ?? '',
      title: json['title'] as String? ?? '',
      summary: json['summary'] as String?,
    );
  }
}

class LearningUnitDto {
  final String id;
  final String skill;
  final String title;
  final int estimatedMinutes;
  final String summary;
  final String contentMarkdown;
  final String? videoUrl;
  final String? videoTitle;
  final List<LessonChapterDto> chapters;
  final bool completed;

  LearningUnitDto({
    required this.id,
    required this.skill,
    required this.title,
    required this.estimatedMinutes,
    required this.summary,
    required this.contentMarkdown,
    this.videoUrl,
    this.videoTitle,
    this.chapters = const [],
    this.completed = false,
  });

  factory LearningUnitDto.fromJson(Map<String, dynamic> json) {
    return LearningUnitDto(
      id: json['id'] as String? ?? '',
      skill: json['skill'] as String? ?? '',
      title: json['title'] as String? ?? '',
      estimatedMinutes: (json['estimated_minutes'] as num?)?.toInt() ?? 30,
      summary: json['summary'] as String? ?? '',
      contentMarkdown: json['content_markdown'] as String? ?? '',
      videoUrl: json['video_url'] as String?,
      videoTitle: json['video_title'] as String?,
      chapters: (json['chapters'] as List<dynamic>?)
              ?.map((e) => LessonChapterDto.fromJson(e as Map<String, dynamic>))
              .toList() ??
          [],
      completed: json['completed'] as bool? ?? false,
    );
  }
}

class LlmConfigDto {
  final String provider;
  final String? baseUrl;
  final String? modelChat;
  final String? modelSearch;
  final String? modelEmbed;
  final bool isShared;
  final bool hasKey;
  final String? maskedKey;
  final bool isInherited;

  LlmConfigDto({
    required this.provider,
    this.baseUrl,
    this.modelChat,
    this.modelSearch,
    this.modelEmbed,
    required this.isShared,
    required this.hasKey,
    this.maskedKey,
    required this.isInherited,
  });

  factory LlmConfigDto.fromJson(Map<String, dynamic> json) {
    return LlmConfigDto(
      provider: json['provider'] as String? ?? '',
      baseUrl: json['base_url'] as String?,
      modelChat: json['model_chat'] as String?,
      modelSearch: json['model_search'] as String?,
      modelEmbed: json['model_embed'] as String?,
      isShared: json['is_shared'] as bool? ?? false,
      hasKey: json['has_key'] as bool? ?? false,
      maskedKey: json['masked_key'] as String?,
      isInherited: json['is_inherited'] as bool? ?? false,
    );
  }
}

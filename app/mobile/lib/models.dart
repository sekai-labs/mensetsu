class WorkspaceDto {
  final String id;
  final String userId;
  final String companyName;
  final String roleTitle;
  final String status;
  final String currentPhase;
  final int progressPercent;
  final DateTime? createdAt;
  final DateTime? updatedAt;

  WorkspaceDto({
    required this.id,
    this.userId = '',
    required this.companyName,
    required this.roleTitle,
    required this.status,
    this.currentPhase = 'ready',
    required this.progressPercent,
    this.createdAt,
    this.updatedAt,
  });

  factory WorkspaceDto.fromJson(Map<String, dynamic> json) {
    return WorkspaceDto(
      id: json['id'] as String? ?? '',
      userId: json['user_id'] as String? ?? '',
      companyName: json['company_name'] as String? ?? '',
      roleTitle: json['role_title'] as String? ?? '',
      status: json['status'] as String? ?? 'ready',
      currentPhase: json['current_phase'] as String? ?? 'ready',
      progressPercent: (json['progress_percent'] as num?)?.toInt() ?? 0,
      createdAt: json['created_at'] != null ? DateTime.tryParse(json['created_at'].toString()) : null,
      updatedAt: json['updated_at'] != null ? DateTime.tryParse(json['updated_at'].toString()) : null,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'user_id': userId,
        'company_name': companyName,
        'role_title': roleTitle,
        'status': status,
        'current_phase': currentPhase,
        'progress_percent': progressPercent,
        if (createdAt != null) 'created_at': createdAt!.toIso8601String(),
        if (updatedAt != null) 'updated_at': updatedAt!.toIso8601String(),
      };
}

class MatchStrengthItem {
  final String skill;
  final String evidenceQuote;
  final String relevance;

  MatchStrengthItem({
    required this.skill,
    required this.evidenceQuote,
    this.relevance = 'high',
  });

  factory MatchStrengthItem.fromDynamic(dynamic data) {
    if (data is Map<String, dynamic>) {
      return MatchStrengthItem(
        skill: data['skill'] as String? ?? '',
        evidenceQuote: data['evidence_quote'] as String? ?? '',
        relevance: data['relevance'] as String? ?? 'high',
      );
    }
    return MatchStrengthItem(skill: data.toString(), evidenceQuote: '');
  }
}

class MatchGapItem {
  final String skill;
  final String requirementQuote;
  final String importance;
  final String recommendedFocus;

  MatchGapItem({
    required this.skill,
    required this.requirementQuote,
    this.importance = 'mandatory',
    this.recommendedFocus = '',
  });

  factory MatchGapItem.fromDynamic(dynamic data) {
    if (data is Map<String, dynamic>) {
      return MatchGapItem(
        skill: data['skill'] as String? ?? '',
        requirementQuote: data['requirement_quote'] as String? ?? '',
        importance: data['importance'] as String? ?? 'mandatory',
        recommendedFocus: data['recommended_focus'] as String? ?? '',
      );
    }
    return MatchGapItem(skill: data.toString(), requirementQuote: '');
  }
}

class MatchAnalysisDto {
  final String workspaceId;
  final double overallMatchRatio;
  final List<dynamic> rawStrengths;
  final List<dynamic> rawGaps;
  final List<MatchStrengthItem> parsedStrengths;
  final List<MatchGapItem> parsedGaps;
  final List<String> risks;

  MatchAnalysisDto({
    required this.workspaceId,
    required this.overallMatchRatio,
    required this.rawStrengths,
    required this.rawGaps,
    List<MatchStrengthItem>? parsedStrengths,
    List<MatchGapItem>? parsedGaps,
    this.risks = const [],
  })  : parsedStrengths = parsedStrengths ?? rawStrengths.map(MatchStrengthItem.fromDynamic).toList(),
        parsedGaps = parsedGaps ?? rawGaps.map(MatchGapItem.fromDynamic).toList();

  List<dynamic> get strengths => rawStrengths;
  List<dynamic> get gaps => rawGaps;

  factory MatchAnalysisDto.fromJson(Map<String, dynamic> json) {
    final rawS = json['strengths'] as List<dynamic>? ?? [];
    final rawG = json['gaps'] as List<dynamic>? ?? [];
    final rList = (json['risks'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [];

    return MatchAnalysisDto(
      workspaceId: json['workspace_id'] as String? ?? '',
      overallMatchRatio: (json['overall_match_ratio'] as num?)?.toDouble() ?? 0.0,
      rawStrengths: rawS,
      rawGaps: rawG,
      risks: rList,
    );
  }
}

class CompanyCitationItem {
  final String sourceUrl;
  final String title;
  final String snippet;

  CompanyCitationItem({
    required this.sourceUrl,
    required this.title,
    required this.snippet,
  });

  factory CompanyCitationItem.fromJson(Map<String, dynamic> json) {
    return CompanyCitationItem(
      sourceUrl: json['source_url'] as String? ?? '',
      title: json['title'] as String? ?? '',
      snippet: json['snippet'] as String? ?? '',
    );
  }
}

class CompanyResearchDto {
  final String workspaceId;
  final String companyName;
  final String overview;
  final List<String> technologyStack;
  final List<String> engineeringCulturePillars;
  final List<CompanyCitationItem> citations;

  CompanyResearchDto({
    required this.workspaceId,
    required this.companyName,
    required this.overview,
    this.technologyStack = const [],
    this.engineeringCulturePillars = const [],
    this.citations = const [],
  });

  factory CompanyResearchDto.fromJson(Map<String, dynamic> json) {
    final stack = (json['technology_stack'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [];
    final culture = (json['engineering_culture_pillars'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [];
    final rawCitations = json['sources'] as List<dynamic>? ?? json['citations'] as List<dynamic>? ?? [];

    return CompanyResearchDto(
      workspaceId: json['workspace_id'] as String? ?? '',
      companyName: json['company_name'] as String? ?? '',
      overview: json['overview'] as String? ?? '',
      technologyStack: stack,
      engineeringCulturePillars: culture,
      citations: rawCitations
          .whereType<Map<String, dynamic>>()
          .map(CompanyCitationItem.fromJson)
          .toList(),
    );
  }
}

class InterviewQuestionDto {
  final String id;
  final String question;
  final String difficulty;
  final String interviewerIntent;
  final List<String> answerOutline;
  final List<String> strongAnswerRubric;
  final List<String> commonWeaknesses;

  InterviewQuestionDto({
    required this.id,
    required this.question,
    required this.difficulty,
    required this.interviewerIntent,
    required this.answerOutline,
    this.strongAnswerRubric = const [],
    this.commonWeaknesses = const [],
  });

  factory InterviewQuestionDto.fromJson(Map<String, dynamic> json) {
    return InterviewQuestionDto(
      id: json['id'] as String? ?? '',
      question: json['question'] as String? ?? '',
      difficulty: json['difficulty'] as String? ?? 'medium',
      interviewerIntent: json['interviewer_intent'] as String? ?? '',
      answerOutline: (json['answer_outline'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
      strongAnswerRubric: (json['strong_answer_rubric'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
      commonWeaknesses: (json['common_weaknesses'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
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

  LearningUnitDto copyWith({bool? completed}) {
    return LearningUnitDto(
      id: id,
      skill: skill,
      title: title,
      estimatedMinutes: estimatedMinutes,
      summary: summary,
      contentMarkdown: contentMarkdown,
      videoUrl: videoUrl,
      videoTitle: videoTitle,
      chapters: chapters,
      completed: completed ?? this.completed,
    );
  }

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
              ?.whereType<Map<String, dynamic>>()
              .map(LessonChapterDto.fromJson)
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

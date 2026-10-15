

enum WorkspaceStatus { pending, processing, ready, failed }

enum PipelinePhase {
  preparingInput,
  analyzingResume,
  readingJobDescription,
  researchingCompany,
  researchingEngineeringEnvironment,
  extractingRoleRequirements,
  mappingResumeEvidence,
  constructingKnowledgeGraph,
  buildingLearningPlan,
  generatingLessons,
  buildingInterviewQuestionBook,
  finalValidation,
  ready,
  failed,
}

class WorkspaceDto {
  final String id;
  final String userId;
  final String companyName;
  final String roleTitle;
  final String status;
  final String currentPhase;
  final int progressPercent;
  final DateTime createdAt;
  final DateTime updatedAt;

  WorkspaceDto({
    required this.id,
    required this.userId,
    required this.companyName,
    required this.roleTitle,
    required this.status,
    required this.currentPhase,
    required this.progressPercent,
    required this.createdAt,
    required this.updatedAt,
  });

  factory WorkspaceDto.fromJson(Map<String, dynamic> json) {
    return WorkspaceDto(
      id: json['id'] as String? ?? '',
      userId: json['user_id'] as String? ?? '',
      companyName: json['company_name'] as String? ?? '',
      roleTitle: json['role_title'] as String? ?? '',
      status: json['status'] as String? ?? 'pending',
      currentPhase: json['current_phase'] as String? ?? 'preparing_input',
      progressPercent: (json['progress_percent'] as num?)?.toInt() ?? 0,
      createdAt: DateTime.tryParse(json['created_at'] as String? ?? '') ?? DateTime.now(),
      updatedAt: DateTime.tryParse(json['updated_at'] as String? ?? '') ?? DateTime.now(),
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
    'created_at': createdAt.toIso8601String(),
    'updated_at': updatedAt.toIso8601String(),
  };
}

class MatchAnalysisDto {
  final String workspaceId;
  final double overallMatchRatio;
  final List<dynamic> strengths;
  final List<dynamic> gaps;
  final List<dynamic> risks;
  final List<dynamic> wordingImprovements;

  MatchAnalysisDto({
    required this.workspaceId,
    required this.overallMatchRatio,
    required this.strengths,
    required this.gaps,
    required this.risks,
    required this.wordingImprovements,
  });

  factory MatchAnalysisDto.fromJson(Map<String, dynamic> json) {
    return MatchAnalysisDto(
      workspaceId: json['workspace_id'] as String? ?? '',
      overallMatchRatio: (json['overall_match_ratio'] as num?)?.toDouble() ?? 0.0,
      strengths: json['strengths'] as List<dynamic>? ?? [],
      gaps: json['gaps'] as List<dynamic>? ?? [],
      risks: json['risks'] as List<dynamic>? ?? [],
      wordingImprovements: json['wording_improvements'] as List<dynamic>? ?? [],
    );
  }
}

class InterviewQuestionDto {
  final String id;
  final String workspaceId;
  final String category;
  final String question;
  final String difficulty;
  final List<String> skillsTested;
  final String interviewerIntent;
  final List<String> answerOutline;
  final List<String> strongAnswerRubric;
  final List<String> commonWeaknesses;
  final List<String> followUpQuestions;

  InterviewQuestionDto({
    required this.id,
    required this.workspaceId,
    required this.category,
    required this.question,
    required this.difficulty,
    required this.skillsTested,
    required this.interviewerIntent,
    required this.answerOutline,
    required this.strongAnswerRubric,
    required this.commonWeaknesses,
    required this.followUpQuestions,
  });

  factory InterviewQuestionDto.fromJson(Map<String, dynamic> json) {
    return InterviewQuestionDto(
      id: json['id'] as String? ?? '',
      workspaceId: json['workspace_id'] as String? ?? '',
      category: json['category'] as String? ?? '',
      question: json['question'] as String? ?? '',
      difficulty: json['difficulty'] as String? ?? 'medium',
      skillsTested: (json['skills_tested'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
      interviewerIntent: json['interviewer_intent'] as String? ?? '',
      answerOutline: (json['answer_outline'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
      strongAnswerRubric: (json['strong_answer_rubric'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
      commonWeaknesses: (json['common_weaknesses'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
      followUpQuestions: (json['follow_up_questions'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? [],
    );
  }
}

import 'dart:convert';
import 'package:http/http.dart' as http;
import 'models.dart';

class MensetsuApiClient {
  final String baseUrl;
  final http.Client client;

  MensetsuApiClient({this.baseUrl = 'http://127.0.0.1:3000', http.Client? client})
      : client = client ?? http.Client();

  String _cleanId(String id) {
    return id.replaceAll('workspace:', '').replaceAll('`', '');
  }

  Future<List<WorkspaceDto>> listWorkspaces() async {
    try {
      final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces'));
      if (response.statusCode == 200) {
        final list = jsonDecode(response.body) as List<dynamic>;
        return list.map((e) => WorkspaceDto.fromJson(e as Map<String, dynamic>)).toList();
      }
    } catch (_) {
    }
    return _mockWorkspaces();
  }

  Future<WorkspaceDto> getWorkspace(String id) async {
    final clean = _cleanId(id);
    try {
      final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$clean'));
      if (response.statusCode == 200) {
        return WorkspaceDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
      }
    } catch (_) {}
    return _mockWorkspaces().firstWhere(
      (w) => _cleanId(w.id) == clean,
      orElse: () => WorkspaceDto(
        id: clean,
        companyName: 'Target Enterprise',
        roleTitle: 'Principal Software Engineer',
        status: 'ready',
        progressPercent: 100,
      ),
    );
  }

  Future<WorkspaceDto> createWorkspace({
    required String companyName,
    required String roleTitle,
    String? rawJobDescription,
    String? rawResume,
  }) async {
    final payload = {
      'company_name': companyName,
      'role_title': roleTitle,
      'raw_job_description': rawJobDescription ?? '',
      'raw_resume': rawResume ?? '',
    };
    try {
      final response = await client.post(
        Uri.parse('$baseUrl/api/v1/workspaces'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(payload),
      );
      if (response.statusCode == 200 || response.statusCode == 201) {
        return WorkspaceDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
      }
    } catch (_) {}
    return WorkspaceDto(
      id: 'ws_${DateTime.now().millisecondsSinceEpoch}',
      companyName: companyName,
      roleTitle: roleTitle,
      status: 'ready',
      progressPercent: 100,
    );
  }

  Future<MatchAnalysisDto> getWorkspaceMatch(String id) async {
    final clean = _cleanId(id);
    try {
      final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$clean/match'));
      if (response.statusCode == 200) {
        return MatchAnalysisDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
      }
    } catch (_) {}
    return _mockMatch(clean);
  }

  Future<CompanyResearchDto> getWorkspaceCompany(String id) async {
    final clean = _cleanId(id);
    try {
      final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$clean/company'));
      if (response.statusCode == 200) {
        return CompanyResearchDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
      }
    } catch (_) {}
    return _mockCompany(clean);
  }

  Future<List<InterviewQuestionDto>> getWorkspaceQuestions(String id) async {
    final clean = _cleanId(id);
    try {
      final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$clean/questions'));
      if (response.statusCode == 200) {
        final list = jsonDecode(response.body) as List<dynamic>;
        return list.map((e) => InterviewQuestionDto.fromJson(e as Map<String, dynamic>)).toList();
      }
    } catch (_) {}
    return _mockQuestions(clean);
  }

  Future<List<LearningUnitDto>> getWorkspaceLearning(String id) async {
    final clean = _cleanId(id);
    try {
      final response = await client.get(Uri.parse('$baseUrl/api/v1/workspaces/$clean/learning'));
      if (response.statusCode == 200) {
        final list = jsonDecode(response.body) as List<dynamic>;
        return list.map((e) => LearningUnitDto.fromJson(e as Map<String, dynamic>)).toList();
      }
    } catch (_) {}
    return _mockLearning(clean);
  }

  Future<bool> toggleLessonCompleted(String workspaceId, String lessonId, {String? token}) async {
    final cleanWs = _cleanId(workspaceId);
    final cleanLid = lessonId.replaceAll('learning_unit:', '').replaceAll('`', '');
    try {
      final response = await client.put(
        Uri.parse('$baseUrl/api/v1/workspaces/$cleanWs/learning/$cleanLid/toggle'),
        headers: {
          'Content-Type': 'application/json',
          if (token != null) 'Authorization': 'Bearer $token',
        },
      );
      if (response.statusCode == 200) {
        final data = jsonDecode(response.body) as Map<String, dynamic>;
        return data['completed'] as bool? ?? true;
      }
    } catch (_) {}
    return true;
  }

  Future<List<LlmConfigDto>> getLlmSettings({String? token}) async {
    try {
      final response = await client.get(
        Uri.parse('$baseUrl/api/v1/settings/llm'),
        headers: {if (token != null) 'Authorization': 'Bearer $token'},
      );
      if (response.statusCode == 200) {
        final list = jsonDecode(response.body) as List<dynamic>;
        return list.map((e) => LlmConfigDto.fromJson(e as Map<String, dynamic>)).toList();
      }
    } catch (_) {}
    return _mockLlmSettings();
  }

  Future<LlmConfigDto> updateLlmSetting(Map<String, dynamic> payload, {String? token}) async {
    try {
      final response = await client.put(
        Uri.parse('$baseUrl/api/v1/settings/llm'),
        headers: {
          'Content-Type': 'application/json',
          if (token != null) 'Authorization': 'Bearer $token',
        },
        body: jsonEncode(payload),
      );
      if (response.statusCode == 200) {
        return LlmConfigDto.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
      }
    } catch (_) {}
    return LlmConfigDto.fromJson(payload);
  }

  static List<WorkspaceDto> _mockWorkspaces() => [
        WorkspaceDto(
          id: 'ws_scalevector',
          companyName: 'ScaleVector Systems',
          roleTitle: 'Staff Systems Architect',
          status: 'ready',
          progressPercent: 100,
          currentPhase: 'ready',
        ),
        WorkspaceDto(
          id: 'ws_cloudnative',
          companyName: 'HyperScale Cloud',
          roleTitle: 'Principal Platform Engineer',
          status: 'ready',
          progressPercent: 100,
          currentPhase: 'ready',
        ),
      ];

  static MatchAnalysisDto _mockMatch(String wsId) => MatchAnalysisDto(
        workspaceId: wsId,
        overallMatchRatio: 0.88,
        rawStrengths: [
          {
            'skill': 'Distributed Systems & Consensus',
            'evidence_quote': 'Authored Raft-based state-machine replication engine in production.',
            'relevance': 'Primary',
          },
          {
            'skill': 'Rust Systems Programming',
            'evidence_quote': 'Maintained core zero-copy I/O networking layer with Tokio.',
            'relevance': 'Primary',
          },
          {
            'skill': 'Database Internals',
            'evidence_quote': 'Optimized LSM-tree compaction and WAL replay latency.',
            'relevance': 'Secondary',
          },
        ],
        rawGaps: [
          {
            'skill': 'eBPF Kernel Tracing',
            'requirement_quote': 'Hands-on experience profiling kernel network bottlenecks via eBPF/BCC.',
            'importance': 'Mandatory',
            'recommended_focus': 'Deep dive into Linux socket filters and XDP packet filtering.',
          },
          {
            'skill': 'Temporal Workflow SDK',
            'requirement_quote': 'Experience orchestrating long-running durable workflows.',
            'importance': 'Preferred',
            'recommended_focus': 'Review workflow determinism invariants and activity retry policies.',
          },
        ],
        risks: [
          'Limited public record on multi-region geo-partitioned failover drills.',
        ],
      );

  static CompanyResearchDto _mockCompany(String wsId) => CompanyResearchDto(
        workspaceId: wsId,
        companyName: 'ScaleVector Systems',
        overview:
            'ScaleVector provides high-throughput, low-latency distributed event orchestration and storage engines for hyper-growth cloud providers. Their stack prioritizes zero-copy architectures, predictability over peak throughput, and strict formal verification.',
        technologyStack: [
          'Rust',
          'Tokio',
          'Linux eBPF',
          'SurrealDB',
          'Temporal',
          'Raft',
          'gRPC',
          'Prometheus',
        ],
        engineeringCulturePillars: [
          'Formal RFCs and design reviews precede all production code.',
          'Postmortems focus on systemic blameless invariants.',
          'P99 latency predictability is valued over raw median operations.',
        ],
        citations: [
          CompanyCitationItem(
            sourceUrl: 'https://github.com/scalevector/architecture-rfcs',
            title: 'ScaleVector Engineering Principles & Distributed RFCs',
            snippet: 'RFC 042: Bounded lease renewal invariants and split-brain fencing tokens.',
          ),
          CompanyCitationItem(
            sourceUrl: 'https://blog.scalevector.io/ebpf-observability',
            title: 'Profiling Production Tail Latency with eBPF and XDP',
            snippet: 'How we reduced p99 packet drop rates by 80% using kernel-level packet inspection.',
          ),
        ],
      );

  static List<InterviewQuestionDto> _mockQuestions(String wsId) => [
        InterviewQuestionDto(
          id: 'q_1',
          question:
              'In your previous systems, how did you prevent duplicate message dispatch, avoid zombie leases, and recover in-flight transactions without losing linearizable state after sudden worker crashes?',
          difficulty: 'Hard',
          interviewerIntent:
              'Evaluates candidate\'s real-world systems debugging experience under dirty crashes and network partitions.',
          answerOutline: [
            '1. Heartbeat mechanisms and bounded lease duration (e.g. 30s TTL with renew interval 10s).',
            '2. Fencing tokens to reject stale writes from zombie nodes after split-brain recovery.',
            '3. Transactional outbox and unique idempotency deduplication keys in storage.',
            '4. Dead letter queue routing with exponential backoff policies for unrecoverable errors.',
          ],
          strongAnswerRubric: [
            'Articulates split-brain scenarios and fencing tokens rather than relying on client wall-clock time.',
            'Demonstrates explicit distinction between at-least-once transport and exactly-once processing via idempotency keys.',
          ],
          commonWeaknesses: [
            'Assuming TCP connection termination automatically notifies the coordinator during network partitions.',
          ],
        ),
        InterviewQuestionDto(
          id: 'q_2',
          question:
              'Design an end-to-end event ingestion and analysis platform that processes 200,000 requests/sec with p99 latency under 20ms and multi-region active-active durability.',
          difficulty: 'Hard',
          interviewerIntent:
              'Probes architectural intuition around network ingress, partitioning, queue backpressure, and cross-region consensus.',
          answerOutline: [
            '1. Clarify ingress requirements, payload sizes, read vs write ratios, and consistency models.',
            '2. Propose Anycast routing, global API gateways, and asynchronous buffering tier.',
            '3. Design consistent hashing key distribution to prevent hotspotting.',
            '4. Outline storage partitioning with WAL replication and multi-tier read caches.',
          ],
          strongAnswerRubric: [
            'Calculates realistic capacity numbers and bandwidth requirements.',
            'Proactively addresses hot partitions and backpressure mitigation.',
          ],
          commonWeaknesses: [
            'Proposing direct synchronous writes to a relational database without buffering.',
          ],
        ),
        InterviewQuestionDto(
          id: 'q_3',
          question:
              'Explain the Linux memory model, page faults, and how memory-mapped I/O (mmap) interacts with the kernel page cache under high read concurrency.',
          difficulty: 'Medium',
          interviewerIntent:
              'Verifies systems fundamentals and low-level understanding of kernel virtual memory management and page cache contention.',
          answerOutline: [
            '1. Explain virtual vs physical memory translation via page tables and TLB caches.',
            '2. Walk through minor vs major page faults and disk reads into kernel page cache.',
            '3. Discuss mmap vs read syscall tradeoffs: avoidance of user/kernel buffer copying.',
          ],
          strongAnswerRubric: [
            'Mentions madvise(MADV_WILLNEED / MADV_RANDOM) hints for prefetching.',
          ],
          commonWeaknesses: [
            'Thinking mmap permanently pins entire file buffers in physical RAM.',
          ],
        ),
      ];

  static List<LearningUnitDto> _mockLearning(String wsId) => [
        LearningUnitDto(
          id: 'lu_1',
          skill: 'Durable Temporal Workflows',
          title: 'Reliable Distributed Orchestration with Temporal',
          estimatedMinutes: 25,
          summary: 'Mastering state-machine execution, event histories, and activity retry invariants.',
          contentMarkdown: '''
### Core Invariants

Temporal workflows must be deterministic. Never introduce non-deterministic I/O directly into workflow functions.

```python
# Invariant: Deterministic workflow code
@workflow.defn
class SafeWorkflow:
    @workflow.run
    async def run(self, input: str) -> str:
        return await workflow.execute_activity(
            safe_activity,
            input,
            start_to_close_timeout=timedelta(seconds=60)
        )
```

#### Key Rules:
1. No thread sleeps (use `workflow.sleep`).
2. No direct network or DB calls (delegate to activities).
3. Always provide idempotency tokens.
''',
          videoUrl: 'https://www.youtube-nocookie.com/embed/2Hk_q0d21e0',
          videoTitle: 'Temporal Architecture & Durable Execution',
          chapters: [
            LessonChapterDto(timestamp: '00:00', title: 'Why Durable Workflows Matter', summary: 'Failure modes of microservices'),
            LessonChapterDto(timestamp: '06:30', title: 'Determinism Invariants', summary: 'Replay safety and event history'),
            LessonChapterDto(timestamp: '14:20', title: 'Activities & Retry Policies', summary: 'Backoff exponent and timeouts'),
          ],
          completed: false,
        ),
        LearningUnitDto(
          id: 'lu_2',
          skill: 'Linux eBPF Tracing',
          title: 'Kernel-Level Observability with eBPF & XDP',
          estimatedMinutes: 35,
          summary: 'Inspect socket buffers and capture networking metrics without runtime performance penalties.',
          contentMarkdown: '''
### eBPF Architecture

Extended Berkeley Packet Filter allows sandboxed programs to execute inside the Linux kernel in response to kernel probe events.

```c
SEC("tracepoint/syscalls/sys_enter_connect")
int trace_connect(struct trace_event_raw_sys_enter *ctx) {
    bpf_printk("Network socket connect initiated\\n");
    return 0;
}
```

#### Verification Guarantees:
- In-kernel verifier ensures programs do not crash or loop infinitely.
- Zero copy inspect of sk_buff before packet reaches userspace socket.
''',
          chapters: [
            LessonChapterDto(timestamp: '00:00', title: 'Kernel Tracing Overview'),
            LessonChapterDto(timestamp: '08:45', title: 'BPF Verifier & Safety'),
            LessonChapterDto(timestamp: '18:10', title: 'XDP High Speed Ingress'),
          ],
          completed: true,
        ),
      ];

  static List<LlmConfigDto> _mockLlmSettings() => [
        LlmConfigDto(
          provider: 'openai',
          modelChat: 'gpt-4o',
          modelSearch: 'gpt-4o-mini',
          modelEmbed: 'text-embedding-3-small',
          isShared: true,
          hasKey: true,
          maskedKey: 'sk-...4921',
          isInherited: false,
        ),
        LlmConfigDto(
          provider: 'anthropic',
          modelChat: 'claude-3-5-sonnet-20241022',
          isShared: false,
          hasKey: true,
          maskedKey: 'sk-ant-...8901',
          isInherited: false,
        ),
        LlmConfigDto(
          provider: 'local-ollama',
          baseUrl: 'http://localhost:11434',
          modelChat: 'llama3.2',
          isShared: false,
          hasKey: false,
          isInherited: false,
        ),
      ];
}

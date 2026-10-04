USE NS mensetsu DB production;

UPSERT user:user_default SET
  email = 'admin@mensetsu.local',
  name = 'Lead Architect',
  password_hash = '$argon2id$v=19$m=19456,t=2,p=1$ZGVtb3NhbHQ$demo_hash_for_testing';

UPSERT user:demo_candidate SET
  email = 'candidate@mensetsu.local',
  name = 'Alex Chen',
  password_hash = '$argon2id$v=19$m=19456,t=2,p=1$ZGVtb3NhbHQ$demo_hash_for_testing';

UPSERT workspace:demo_cloudflare SET
  user_id = user:demo_candidate,
  company_name = 'Cloudflare',
  role_title = 'Staff Distributed Systems Engineer',
  status = 'ready',
  current_phase = 'completed',
  progress_percent = 100,
  seniority = 'staff',
  prep_time_weeks = 4;

UPSERT workspace:demo_stripe SET
  user_id = user:demo_candidate,
  company_name = 'Stripe',
  role_title = 'Principal Infrastructure Engineer',
  status = 'ready',
  current_phase = 'completed',
  progress_percent = 100,
  seniority = 'principal',
  prep_time_weeks = 6;

UPSERT company_entity:cloudflare SET
  name = 'Cloudflare',
  domain = 'cloudflare.com',
  overview = 'Global network cloud platform providing edge computing, CDN, DDoS mitigation, and Workers serverless runtime.',
  business_model = 'Usage-based enterprise edge compute, security subscriptions, and developer platform tier';

UPSERT source:cloudflare_blog_1 SET
  workspace_id = workspace:demo_cloudflare,
  url = 'https://blog.cloudflare.com/how-we-built-pingora-in-rust',
  title = 'How we built Pingora, the proxy that powers Cloudflare',
  domain = 'blog.cloudflare.com',
  snippet = 'Pingora is an async Rust multi-threaded proxy that replaced NGINX, handling over one trillion requests daily with 70% less CPU.',
  source_type = 'engineering_blog',
  confidence = 0.98;

UPSERT source:cloudflare_blog_2 SET
  workspace_id = workspace:demo_cloudflare,
  url = 'https://blog.cloudflare.com/workers-under-the-hood',
  title = 'How Cloudflare Workers works under the hood',
  domain = 'blog.cloudflare.com',
  snippet = 'V8 isolates run thousands of customer applications within a single OS process, achieving sub-millisecond cold starts.',
  source_type = 'engineering_blog',
  confidence = 0.95;

UPSERT match_analysis:cloudflare_match SET
  workspace_id = workspace:demo_cloudflare,
  overall_match_ratio = 0.91,
  strengths = [
    {
      skill: 'Rust Async Systems Programming',
      evidence_quote: 'Designed and implemented high-throughput Tokio async network engine processing 100k QPS.',
      relevance: 'Direct alignment with Pingora edge service and custom zero-copy buffer pools.'
    },
    {
      skill: 'Distributed Consensus & Raft',
      evidence_quote: 'Engineered Raft-based replicated metadata log with automated leader recovery.',
      relevance: 'Applicable to Cloudflare global KV replication and cluster state consensus.'
    }
  ],
  gaps = [
    {
      skill: 'eBPF / XDP Network Ingestion',
      missing_context: 'Resume emphasizes user-space TCP sockets; lacks production kernel bypass eBPF hooks.',
      suggested_action: 'Study Linux XDP driver hooks, TC BPF filters, and Cilium eBPF pipeline.'
    }
  ],
  risks = [
    'Absence of production kernel-bypass (DPDK/XDP) scale experience.'
  ],
  wording_improvements = [
    {
      original_quote: 'Handled proxy server maintenance',
      suggested_revision: 'Architected non-blocking reverse proxy runtime leveraging Epoll and Tokio task pools',
      impact: 'Highlights low-latency systems engineering rigor'
    }
  ];

UPSERT learning_unit:cf_lesson_1 SET
  workspace_id = workspace:demo_cloudflare,
  skill = 'eBPF Kernel Bypass',
  title = 'eBPF and XDP Architecture at the Edge',
  prerequisites = ['Linux networking', 'BPF bytecode verification'],
  estimated_minutes = 25,
  summary = 'Deep dive into XDP packet filter hooks running directly within NIC driver buffers.',
  content_markdown = 'XDP (eXpress Data Path) executes verified eBPF bytecode inside the device driver before kernel sk_buff allocation, delivering 10M+ PPS line-rate filtering.',
  diagram = {
    diagram_type: 'architecture_diagram',
    caption: 'XDP Packet Ingestion Pipeline',
    explanation: 'NIC Ring Buffer -> Driver Hook -> TCP/IP Stack',
    nodes: [
      { id: 'nic', label: 'NIC Hardware RX Ring', type: 'hardware' },
      { id: 'xdp', label: 'XDP Driver Hook (eBPF)', type: 'kernel' },
      { id: 'stack', label: 'Linux Network Stack', type: 'userspace' }
    ],
    edges: [
      { from: 'nic', to: 'xdp', relation: 'rx_packet' },
      { from: 'xdp', to: 'stack', relation: 'pass_allowed' }
    ]
  };

UPSERT interview_question:cf_q_1 SET
  workspace_id = workspace:demo_cloudflare,
  category = 'system_design',
  question = 'How would you architect a global edge cache that invalidates distributed state in sub-100ms?',
  difficulty = 'hard',
  skills_tested = ['Cache Consistency', 'Anycast Routing', 'Distributed Systems'],
  interviewer_intent = 'Examine tradeoffs between eventual consistency, quorum invalidations, and gossip propagation.',
  answer_outline = [
    'Define Anycast BGP routing topology directing traffic to nearest PoP',
    'Tiered cache hierarchy: L1 local V8 memory, L2 metro NVMe pool, L3 origin',
    'Pub/Sub purge fanout via QUIC streams to all global PoPs',
    'Single-flight mutex synchronization to prevent origin thundering herd'
  ],
  strong_answer_rubric = [
    'Mentions single-flight deduplication on cache miss stampedes',
    'Analyzes consistency vs latency tradeoffs in two-phase commit vs gossip'
  ],
  common_weaknesses = [
    'Relying on a single centralized Redis instance globally',
    'Ignoring clock drift across geographic regions'
  ],
  follow_up_questions = [
    'What happens if a purge event is dropped by a transient network partition?',
    'How do you handle purges on wildcard domain tags?'
  ];

UPSERT interview_question:cf_q_2 SET
  workspace_id = workspace:demo_cloudflare,
  category = 'technical_fundamentals',
  question = 'Explain memory safety guarantees and zero-cost abstractions in Rust when managing raw socket buffers.',
  difficulty = 'hard',
  skills_tested = ['Rust', 'Memory Management', 'Async I/O'],
  interviewer_intent = 'Evaluate deep understanding of ownership, lifetimes, Pin, and uninitialized memory safety.',
  answer_outline = [
    'Ownership model and affine types preventing double-free and use-after-free',
    'Pin and Unpin mechanics in async Future state machines',
    'Zero-copy slice borrowing from ring buffers using scoped lifetimes'
  ],
  strong_answer_rubric = [
    'Articulates how Pin ensures self-referential generator safety',
    'Differentiates MaybeUninit from uninitialized raw pointers'
  ],
  common_weaknesses = [
    'Assuming clone() is costless for heap buffers',
    'Inability to explain why async functions require Pin'
  ],
  follow_up_questions = [
    'How does Tokio manage work-stealing across thread pool runtimes?'
  ];


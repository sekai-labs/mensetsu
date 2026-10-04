USE NS mensetsu DB production;

CREATE user:demo_user SET
  email = 'candidate@sekai.local',
  name = 'Demo Candidate',
  password_hash = 'argon2_dummy_hash';

CREATE workspace:demo_ws SET
  user_id = user:demo_user,
  company_name = 'Cloudflare',
  role_title = 'Senior Distributed Systems Engineer',
  status = 'ready',
  current_phase = 'completed',
  progress_percent = 100,
  seniority = 'senior',
  prep_time_weeks = 4;

CREATE company_entity:demo_company SET
  name = 'Cloudflare',
  domain = 'cloudflare.com',
  overview = 'Cloudflare builds edge infrastructure, CDN, DDoS mitigation, and Workers distributed serverless platform.',
  business_model = 'Usage-based enterprise edge computing and developer subscriptions';

CREATE source:demo_source_1 SET
  workspace_id = workspace:demo_ws,
  url = 'https://blog.cloudflare.com/workers-under-the-hood',
  title = 'How Cloudflare Workers works under the hood',
  domain = 'blog.cloudflare.com',
  snippet = 'V8 isolates provide lightweight sandboxing without container overhead, reducing startup latency to milliseconds.',
  source_type = 'engineering_blog',
  confidence = 0.96;

CREATE match_analysis:demo_match SET
  workspace_id = workspace:demo_ws,
  overall_match_ratio = 0.88,
  strengths = [
    {
      skill: 'Rust Systems Programming',
      evidence_quote: 'Built async Tokio-based proxy service handling 50k RPS in production',
      relevance: 'Direct match for Cloudflare edge proxy and Pingora architecture'
    },
    {
      skill: 'Distributed Consensus & Raft',
      evidence_quote: 'Implemented leader election and distributed log replication in Raft',
      relevance: 'Applies to metadata management and KV global consistency'
    }
  ],
  gaps = [
    {
      skill: 'eBPF / XDP Packet Filtering',
      missing_context: 'No direct XDP experience cited; background is primarily user-space networking',
      importance: 'mandatory',
      suggested_action: 'Study Linux XDP driver hooks and Cilium packet flow'
    }
  ],
  risks = [
    'Candidate has not worked with kernel bypass networking (DPDK/XDP) in production scale.'
  ],
  wording_improvements = [
    {
      original_quote: 'Worked on networking stack',
      suggested_revision: 'Architected high-throughput non-blocking TCP socket engine using Epoll and Tokio',
      impact: 'Highlights concurrency mastery and high-performance ergonomics'
    }
  ];

CREATE learning_unit:demo_learn_1 SET
  workspace_id = workspace:demo_ws,
  skill = 'eBPF Kernel Bypass',
  title = 'eBPF and XDP Architecture at the Edge',
  prerequisites = ['Linux networking', 'BPF bytecode verification'],
  estimated_minutes = 20,
  summary = 'Deep dive into how XDP packet filters drop attacks before sk_buff allocation.',
  content_markdown = 'XDP (eXpress Data Path) executes eBPF bytecode directly inside the network driver before sk_buff memory allocation, giving raw line-rate performance.',
  diagram = {
    diagram_type: 'architecture_diagram',
    caption: 'XDP Packet Ingestion Pipeline',
    explanation: 'NIC Ring Buffer to Driver Hook to TCP/IP Stack',
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

CREATE interview_question:demo_q_1 SET
  workspace_id = workspace:demo_ws,
  category = 'system_design',
  question = 'How would you architect a global edge cache that invalidates distributed state in sub-100ms?',
  difficulty = 'hard',
  skills_tested = ['Cache Consistency', 'Anycast Routing', 'Distributed Systems'],
  interviewer_intent = 'Examine tradeoff choices between eventual consistency, quorum invalidations, and gossip propagation.',
  answer_outline = [
    'Define Anycast routing topology to nearest PoP',
    'Tiered cache hierarchy: L1 in-memory local, L2 distributed SSD per metro, L3 global origin',
    'Pub/Sub event fanout for purge invalidation tokens via QUIC',
    'Failure modes: Split brain, origin stampede mitigation with single-flight mutex'
  ],
  strong_answer_rubric = [
    'Explicitly mentions single-flight cache loaders to prevent thundering herd',
    'Discusses two-phase commit vs gossip protocol latency tradeoffs'
  ],
  common_weaknesses = [
    'Relying on a single centralized Redis cluster globally',
    'Assuming zero clock drift across edge nodes'
  ],
  follow_up_questions = [
    'What happens if a purge message is dropped by a transient partition?',
    'How do you handle purges on wildcard tag domains?'
  ];

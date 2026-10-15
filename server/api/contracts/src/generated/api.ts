export interface paths {
    "/api/v1/auth/2fa/disable": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["disable_2fa"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/2fa/setup": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["setup_2fa"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/2fa/verify": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["verify_2fa"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/login": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["login"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/login-2fa": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["login_2fa"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/logout": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["logout"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/logout-all": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["logout_all"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/me": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["me"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/refresh": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["refresh"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/auth/register": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["register"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/settings/llm": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_llm_settings"];
        put: operations["upsert_llm_setting"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/settings/llm/{provider}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["delete_llm_setting"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/settings/profile": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_profile"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_workflows"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_workflow_runs"];
        put?: never;
        post: operations["trigger_workflow_run"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/agents": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_workflow_run_agents"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/agents/{execution_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_agent"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/agents/{execution_id}/tools": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_workflow_run_agent_tools"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/artifacts": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_artifacts"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/cancel": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["cancel_workflow_run"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/delegations": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_delegations"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/delegations/graph": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_delegation_graph"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/events": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["workflow_run_events"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/events/history": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_events_history"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/graph": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_graph"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/logs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_logs"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/tasks/{task_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_task"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workflows/runs/{run_id}/tools/{tool_invocation_id}/output": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workflow_run_tool_output"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workspaces": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_workspaces"];
        put?: never;
        post: operations["create_workspace"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workspaces/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_workspace"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/v1/workspaces/{id}/learning/{lesson_id}/toggle": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put: operations["toggle_learning_unit_completed"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        AgentExecution: {
            agent_id: string;
            artifact_ids?: string[];
            budgets?: components["schemas"]["ExecutionBudgets"];
            budgets_used?: components["schemas"]["ExecutionBudgetsUsed"];
            child_execution_ids?: string[];
            completed_at?: string | null;
            current_activity?: string | null;
            /** Format: int64 */
            delegation_depth?: number;
            error?: string | null;
            id: string;
            model_name?: string;
            model_provider?: string;
            notes?: components["schemas"]["ExecutionNote"][];
            objective?: string;
            parent_execution_id?: string | null;
            public_summary?: string;
            /** Format: int64 */
            retries?: number;
            run_id: string;
            started_at: string;
            status?: components["schemas"]["ExecutionStatus"];
            task_id: string;
            /** Format: int64 */
            tool_call_count?: number;
        };
        Artifact: {
            agent_execution_id?: string | null;
            content?: components["schemas"]["ArtifactContent"];
            created_at: string;
            evidence?: null | components["schemas"]["EvidenceProvenance"];
            evidence_ids?: string[];
            id: string;
            run_id: string;
            source_links?: components["schemas"]["SourceLink"][];
            surreal_record_id?: string | null;
            task_id?: string | null;
            title: string;
            type: components["schemas"]["ArtifactType"];
            /** Format: int64 */
            version?: number;
        };
        ArtifactContent: Record<string, never> | string;
        /** @enum {string} */
        ArtifactType: "research_report" | "finding" | "evidence" | "question_set" | "recommendation" | "validation_result" | "routing_decision";
        AuthResponseDto: {
            access_token: string;
            refresh_token: string;
            user: components["schemas"]["UserResponseDto"];
        };
        CompanyClaim: {
            claim_type: string;
            /** Format: float */
            confidence: number;
            id: string;
            source_id: string;
            statement: string;
            workspace_id: components["schemas"]["WorkspaceId"];
        };
        CompanyResearch: {
            business_model: string;
            company_name: string;
            engineering_culture: string[];
            interview_process_insights: string[];
            overview: string;
            recent_developments: string[];
            sources: components["schemas"]["SourceCitation"][];
            technology_stack: string[];
            workspace_id: components["schemas"]["WorkspaceId"];
        };
        CreateWorkspaceRequestDto: {
            company_name?: string | null;
            company_urls?: string[];
            jd_url?: string | null;
            job_description?: string | null;
            /** Format: int32 */
            prep_time_weeks?: number | null;
            prompt?: string | null;
            resume_filename?: string | null;
            resume_pdf_base64?: string | null;
            resume_text?: string | null;
            role_title?: string | null;
            seniority?: null | components["schemas"]["SeniorityLevel"];
            /** Format: date-time */
            target_date?: string | null;
            workflow_mode?: string | null;
        };
        Delegation: {
            child_agent_id: string;
            child_execution_id: string;
            completed_at?: string | null;
            created_at: string;
            delegation_task_id: string;
            /** Format: int64 */
            depth?: number;
            error?: string | null;
            id: string;
            input_artifact_ids?: string[];
            objective: string;
            output_artifact_ids?: string[];
            parent_agent_id: string;
            parent_execution_id: string;
            reason: string;
            run_id: string;
            status?: components["schemas"]["DelegationStatus"];
        };
        /** @enum {string} */
        DelegationEdgeKind: "delegation";
        DelegationGraphEdge: {
            data: components["schemas"]["DelegationGraphEdgeData"];
            id: string;
            kind?: components["schemas"]["DelegationEdgeKind"];
            source: string;
            target: string;
        };
        DelegationGraphEdgeData: {
            child_agent_id: string;
            created_at: string;
            delegation_id: string;
            objective: string;
            parent_agent_id: string;
            status: components["schemas"]["DelegationStatus"];
        };
        DelegationGraphNode: {
            data: components["schemas"]["DelegationGraphNodeData"];
            id: string;
            label: string;
            type?: string;
        };
        DelegationGraphNodeData: {
            agent_id: string;
            status: components["schemas"]["ExecutionStatus"];
            task_id: string;
        };
        DelegationGraphResponse: {
            edges: components["schemas"]["DelegationGraphEdge"][];
            nodes: components["schemas"]["DelegationGraphNode"][];
        };
        /** @enum {string} */
        DelegationStatus: "requested" | "accepted" | "running" | "completed" | "failed" | "rejected";
        /** @enum {string} */
        DependencyEdgeKind: "dependency";
        /** @enum {string} */
        ErrorCategory: "network" | "rate_limit" | "budget_exceeded" | "llm_error" | "validation_error" | "internal_error";
        EventsHistoryPageResponse: {
            events: components["schemas"]["ExecutionEvent"][];
            /** Format: int64 */
            last_seq?: number;
            /** Format: int64 */
            next_after_seq?: number | null;
        };
        EvidenceProvenance: {
            /** Format: double */
            confidence: number;
            retrieved_at?: string | null;
            snippet: string;
            source_url: string;
            title: string;
        };
        ExecutionBudgets: {
            /**
             * Format: int64
             * @default 25
             */
            external_requests: number;
            /**
             * Format: int64
             * @default 10
             */
            model_calls: number;
            /**
             * Format: int64
             * @default 50000
             */
            tokens_in: number;
            /**
             * Format: int64
             * @default 10000
             */
            tokens_out: number;
            /**
             * Format: int64
             * @default 20
             */
            tool_calls: number;
            /**
             * Format: double
             * @default 120000
             */
            wall_time_ms: number;
        };
        ExecutionBudgetsUsed: {
            /**
             * Format: int64
             * @default 0
             */
            external_requests: number;
            /**
             * Format: int64
             * @default 0
             */
            model_calls: number;
            /**
             * Format: int64
             * @default 0
             */
            tokens_in: number;
            /**
             * Format: int64
             * @default 0
             */
            tokens_out: number;
            /**
             * Format: int64
             * @default 0
             */
            tool_calls: number;
            /**
             * Format: double
             * @default 0
             */
            wall_time_ms: number;
        };
        ExecutionEvent: {
            correlation_id?: string | null;
            event_id: string;
            execution_id?: string | null;
            parent_execution_id?: string | null;
            payload: Record<string, never>;
            run_id: string;
            schema_version?: string;
            /** Format: int64 */
            seq: number;
            task_id?: string | null;
            tool_invocation_id?: string | null;
            trace_id?: string | null;
            ts: string;
            type: components["schemas"]["ExecutionEventType"];
        };
        /** @enum {string} */
        ExecutionEventType: "agent.activity" | "agent.completed" | "agent.failed" | "agent.started" | "artifact.created" | "checkpoint.created" | "delegation.accepted" | "delegation.completed" | "delegation.failed" | "delegation.requested" | "delegation.started" | "evidence.saved" | "execution.resumed" | "finding.saved" | "plan.created" | "plan.revised" | "task.added" | "task.blocked" | "task.completed" | "task.failed" | "task.queued" | "task.removed" | "task.retrying" | "task.started" | "tool.completed" | "tool.failed" | "tool.started" | "worker.recovered" | "workflow.cancelled" | "workflow.completed" | "workflow.created" | "workflow.failed" | "workflow.retrying" | "workflow.started";
        ExecutionNote: {
            text: string;
            ts: string;
        };
        /** @enum {string} */
        ExecutionStatus: "pending" | "running" | "completed" | "failed" | "cancelled";
        GapEvidence: {
            importance: string;
            recommended_focus: string;
            requirement_quote: string;
            skill: string;
        };
        InterviewQuestion: {
            answer_outline: string[];
            category: components["schemas"]["QuestionCategory"];
            common_weaknesses: string[];
            /** Format: date-time */
            created_at: string;
            difficulty: components["schemas"]["QuestionDifficulty"];
            follow_up_questions: string[];
            id: string;
            interviewer_intent: string;
            question: string;
            skills_tested: string[];
            strong_answer_rubric: string[];
            workspace_id: components["schemas"]["WorkspaceId"];
        };
        KnowledgeEdge: {
            relation: string;
            source: string;
            target: string;
        };
        KnowledgeGraphResponse: {
            edges: components["schemas"]["KnowledgeEdge"][];
            nodes: components["schemas"]["KnowledgeSkillNode"][];
        };
        KnowledgeSkillNode: {
            candidate_status: string;
            category: string;
            id: string;
            importance: string;
            name: string;
            slug: string;
        };
        LearningUnit: {
            chapters?: components["schemas"]["LessonChapter"][] | null;
            completed?: boolean;
            content_markdown: string;
            /** Format: date-time */
            created_at: string;
            diagram?: null | components["schemas"]["VisualDiagram"];
            /** Format: int32 */
            estimated_minutes: number;
            id: string;
            prerequisites: string[];
            skill: string;
            summary: string;
            title: string;
            video_title?: string | null;
            video_url?: string | null;
            workspace_id: components["schemas"]["WorkspaceId"];
        };
        LessonChapter: {
            content?: string | null;
            focus_area?: string | null;
            id?: string | null;
            summary?: string | null;
            timestamp: string;
            title: string;
        };
        LlmConfigItemDto: {
            base_url?: string | null;
            has_key: boolean;
            is_inherited: boolean;
            is_shared: boolean;
            masked_key?: string | null;
            model_chat?: string | null;
            model_embed?: string | null;
            model_search?: string | null;
            provider: string;
        };
        LogEntry: {
            attributes?: Record<string, never>;
            component?: string;
            /** Format: double */
            duration_ms?: number | null;
            error_category?: null | components["schemas"]["ErrorCategory"];
            execution_id?: string | null;
            id: string;
            message: string;
            run_id: string;
            /** Format: int64 */
            seq?: number | null;
            severity?: components["schemas"]["LogSeverity"];
            task_id?: string | null;
            tool_invocation_id?: string | null;
            trace_id?: string | null;
            ts: string;
        };
        /** @enum {string} */
        LogSeverity: "debug" | "info" | "warn" | "error";
        Login2faRequestDto: {
            code: string;
            temp_token: string;
        };
        LoginRequestDto: {
            email: string;
            password: string;
        };
        LoginResponseDto: {
            access_token?: string | null;
            refresh_token?: string | null;
            requires_2fa?: boolean;
            temp_token?: string | null;
            user?: null | components["schemas"]["UserResponseDto"];
        };
        LogsPageResponse: {
            items: components["schemas"]["LogEntry"][];
            next_cursor?: string | null;
        };
        MatchAnalysis: {
            gaps: components["schemas"]["GapEvidence"][];
            /** Format: float */
            overall_match_ratio: number;
            risks: string[];
            strengths: components["schemas"]["StrengthEvidence"][];
            wording_improvements: components["schemas"]["WordingImprovement"][];
            workspace_id: components["schemas"]["WorkspaceId"];
        };
        MessageResponseDto: {
            message: string;
        };
        /** @enum {string} */
        PipelinePhase: "preparing_input" | "analyzing_resume" | "reading_job_description" | "researching_company" | "researching_engineering_environment" | "extracting_role_requirements" | "mapping_resume_evidence" | "constructing_knowledge_graph" | "building_learning_plan" | "generating_lessons" | "building_interview_question_book" | "final_validation" | "ready" | "failed";
        /** @enum {string} */
        QuestionCategory: "resume_deep_dive" | "project_deep_dive" | "technical_fundamentals" | "target_stack" | "system_design" | "debugging" | "behavioral";
        /** @enum {string} */
        QuestionDifficulty: "easy" | "medium" | "hard";
        RefreshTokenRequestDto: {
            refresh_token?: string | null;
        };
        RegisterRequestDto: {
            email: string;
            name: string;
            password: string;
        };
        /** @enum {string} */
        RunStatus: "pending" | "running" | "completed" | "failed" | "cancelled";
        RunsPageResponse: {
            items: components["schemas"]["WorkflowRun"][];
            next_cursor?: string | null;
        };
        /** @enum {string} */
        SeniorityLevel: "junior" | "mid" | "senior" | "staff" | "principal" | "lead";
        SessionId: string;
        SourceCitation: {
            /** Format: float */
            confidence: number;
            domain: string;
            id: string;
            /** Format: date-time */
            retrieved_at: string;
            snippet: string;
            source_type: string;
            title: string;
            url: string;
            workspace_id: components["schemas"]["WorkspaceId"];
        };
        SourceLink: {
            title: string;
            url: string;
        };
        StrengthEvidence: {
            evidence_quote: string;
            relevance: string;
            skill: string;
        };
        TaskGraphEdge: {
            id: string;
            kind?: components["schemas"]["DependencyEdgeKind"];
            source: string;
            target: string;
        };
        TaskGraphNode: {
            data: components["schemas"]["TaskGraphNodeData"];
            id: string;
            label: string;
            type?: string;
        };
        TaskGraphNodeData: {
            /** Format: int64 */
            added_in_revision: number;
            assigned_agent: string;
            /** Format: int64 */
            attempt: number;
            execution_ids?: string[];
            group?: string;
            required: boolean;
            status: components["schemas"]["TaskStatus"];
            task_type: string;
        };
        TaskGraphResponse: {
            edges: components["schemas"]["TaskGraphEdge"][];
            nodes: components["schemas"]["TaskGraphNode"][];
        };
        /** @enum {string} */
        TaskStatus: "pending" | "queued" | "running" | "completed" | "failed" | "retrying" | "blocked" | "cancelled" | "skipped";
        TokenResponseDto: {
            access_token: string;
            refresh_token: string;
        };
        ToolInvocation: {
            completed_at?: string | null;
            /** Format: double */
            duration_ms?: number;
            error?: string | null;
            execution_id: string;
            id: string;
            output_ref?: string | null;
            output_summary?: string;
            sanitized_input?: Record<string, never>;
            started_at: string;
            status?: components["schemas"]["ToolStatus"];
            tool_name: string;
        };
        ToolInvocationsPageResponse: {
            items: components["schemas"]["ToolInvocation"][];
            next_cursor?: string | null;
        };
        ToolOutputResponse: {
            content: string;
            has_more: boolean;
            /** Format: int64 */
            limit: number;
            /** Format: int64 */
            offset: number;
            tool_invocation_id: string;
            /** Format: int64 */
            total_bytes: number;
        };
        /** @enum {string} */
        ToolStatus: "pending" | "running" | "completed" | "failed";
        TotpSetupResponseDto: {
            otpauth_url: string;
            qr_code_data_url: string;
            secret: string;
        };
        TriggerWorkflowRunRequestDto: {
            workflow_id: string;
            workspace_id: string;
        };
        UpsertLlmConfigRequestDto: {
            api_key?: string | null;
            base_url?: string | null;
            is_shared?: boolean | null;
            model_chat?: string | null;
            model_embed?: string | null;
            model_search?: string | null;
            provider: string;
        };
        User: {
            /** Format: date-time */
            created_at: string;
            email: string;
            id: components["schemas"]["UserId"];
            name: string;
            role: components["schemas"]["UserRole"];
            status: components["schemas"]["UserStatus"];
            totp_enabled: boolean;
            /** Format: date-time */
            updated_at: string;
        };
        UserId: string;
        UserProfileResponseDto: {
            email: string;
            id: string;
            name: string;
            role: components["schemas"]["UserRole"];
        };
        UserResponseDto: {
            /** Format: date-time */
            created_at: string;
            email: string;
            id: string;
            name: string;
            role: components["schemas"]["UserRole"];
            status: components["schemas"]["UserStatus"];
            totp_enabled: boolean;
            /** Format: date-time */
            updated_at: string;
        };
        /** @enum {string} */
        UserRole: "admin" | "member";
        /** @enum {string} */
        UserStatus: "active" | "suspended";
        VerifyTotpRequestDto: {
            code: string;
        };
        VisualDiagram: {
            caption: string;
            data: unknown;
            type: string;
        };
        WordingImprovement: {
            original_text: string;
            rationale: string;
            suggested_revision: string;
        };
        WorkflowDefinition: {
            /** Format: int64 */
            default_max_retries?: number;
            description: string;
            id: string;
            name: string;
            supported_agents?: string[];
            type: components["schemas"]["WorkflowType"];
            version?: string;
        };
        WorkflowProgress: {
            /**
             * Format: int64
             * @default 0
             */
            blocked: number;
            /**
             * Format: int64
             * @default 0
             */
            cancelled: number;
            /**
             * Format: int64
             * @default 0
             */
            completed: number;
            /** @default false */
            estimate: boolean;
            /**
             * Format: int64
             * @default 0
             */
            failed: number;
            /**
             * Format: int64
             * @default 0
             */
            pending: number;
            /**
             * Format: int64
             * @default null
             */
            percentage: number | null;
            /**
             * Format: int64
             * @default null
             */
            phase_count: number | null;
            /**
             * Format: int64
             * @default null
             */
            phase_index: number | null;
            /**
             * Format: int64
             * @default 0
             */
            queued: number;
            /**
             * Format: int64
             * @default 0
             */
            retrying: number;
            /**
             * Format: int64
             * @default 0
             */
            running: number;
            /**
             * Format: int64
             * @default 0
             */
            skipped: number;
            /**
             * Format: int64
             * @default 0
             */
            total_tasks: number;
        };
        WorkflowRetryingPayload: {
            /** Format: int64 */
            attempt: number;
            /** Format: int64 */
            delay_ms: number;
            error_category: string;
            /** Format: int64 */
            next_attempt: number;
        };
        WorkflowRun: {
            completed_at?: string | null;
            created_at: string;
            current_phase?: string;
            degraded?: boolean;
            degraded_reasons?: string[];
            /** Format: double */
            duration_ms?: number | null;
            error_message?: string | null;
            id: string;
            /** Format: int64 */
            last_seq?: number;
            /** Format: int64 */
            plan_revision?: number;
            progress?: components["schemas"]["WorkflowProgress"];
            started_at?: string | null;
            status: components["schemas"]["RunStatus"];
            type: components["schemas"]["WorkflowType"];
            updated_at: string;
            user_id: string;
            workflow_id: string;
            workflow_name: string;
            workspace_id: string;
        };
        WorkflowTask: {
            /** Format: int64 */
            added_in_revision?: number;
            assigned_agent: string;
            /** Format: int64 */
            attempt?: number;
            created_at: string;
            dependencies?: string[];
            error?: string | null;
            execution_ids?: string[];
            id: string;
            input_data?: Record<string, never>;
            name: string;
            output_data?: Record<string, never>;
            required?: boolean;
            run_id: string;
            status?: components["schemas"]["TaskStatus"];
            task_type: string;
            updated_at: string;
        };
        /** @enum {string} */
        WorkflowType: "static" | "dynamic" | "hybrid";
        Workspace: {
            company_name: string;
            /** Format: date-time */
            created_at: string;
            current_objective?: string | null;
            current_phase: components["schemas"]["PipelinePhase"];
            id: components["schemas"]["WorkspaceId"];
            /** Format: int32 */
            prep_time_weeks?: number | null;
            /** Format: int32 */
            progress_percent: number;
            role_title: string;
            seniority?: null | components["schemas"]["SeniorityLevel"];
            status: components["schemas"]["WorkspaceStatus"];
            /** Format: date-time */
            target_date?: string | null;
            /** Format: date-time */
            updated_at: string;
            user_id: components["schemas"]["UserId"];
            user_prompt?: string | null;
            workflow_mode: string;
        };
        WorkspaceId: string;
        WorkspaceProgressEventDto: {
            message: string;
            /** Format: int32 */
            percent: number;
            phase: components["schemas"]["PipelinePhase"];
            /** Format: date-time */
            timestamp: string;
            workspace_id: string;
        };
        WorkspaceResponseDto: {
            company_name: string;
            /** Format: date-time */
            created_at: string;
            current_objective?: string | null;
            current_phase: components["schemas"]["PipelinePhase"];
            id: string;
            /** Format: int32 */
            progress_percent: number;
            role_title: string;
            status: components["schemas"]["WorkspaceStatus"];
            /** Format: date-time */
            updated_at: string;
            user_id: string;
            user_prompt?: string | null;
            workflow_mode?: string | null;
        };
        /** @enum {string} */
        WorkspaceStatus: "pending" | "processing" | "ready" | "failed";
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    disable_2fa: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["VerifyTotpRequestDto"];
            };
        };
        responses: {
            /** @description Disable 2FA */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UserResponseDto"];
                };
            };
            /** @description Invalid code */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    setup_2fa: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Initiate 2FA setup and get QR code */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TotpSetupResponseDto"];
                };
            };
            /** @description Unauthorized */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    verify_2fa: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["VerifyTotpRequestDto"];
            };
        };
        responses: {
            /** @description Verify code and activate 2FA */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UserResponseDto"];
                };
            };
            /** @description Invalid code */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    login: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["LoginRequestDto"];
            };
        };
        responses: {
            /** @description Login response or 2FA challenge */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LoginResponseDto"];
                };
            };
            /** @description Invalid credentials or account suspended */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    login_2fa: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["Login2faRequestDto"];
            };
        };
        responses: {
            /** @description Authenticated with 2FA */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AuthResponseDto"];
                };
            };
            /** @description Invalid or expired 2FA code */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    logout: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Logged out successfully */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MessageResponseDto"];
                };
            };
        };
    };
    logout_all: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Logged out from all sessions */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MessageResponseDto"];
                };
            };
        };
    };
    me: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Current authenticated user profile */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UserResponseDto"];
                };
            };
            /** @description Unauthorized */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    refresh: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: {
            content: {
                "application/json": null | components["schemas"]["RefreshTokenRequestDto"];
            };
        };
        responses: {
            /** @description Token refreshed */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TokenResponseDto"];
                };
            };
            /** @description Invalid or expired refresh token */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    register: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["RegisterRequestDto"];
            };
        };
        responses: {
            /** @description Registration successful */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AuthResponseDto"];
                };
            };
            /** @description Validation error */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Email already registered */
            409: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_llm_settings: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description List of configured LLM providers */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LlmConfigItemDto"][];
                };
            };
            /** @description Unauthorized */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    upsert_llm_setting: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpsertLlmConfigRequestDto"];
            };
        };
        responses: {
            /** @description LLM provider settings updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LlmConfigItemDto"];
                };
            };
            /** @description Invalid payload */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Forbidden to share configuration */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    delete_llm_setting: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                provider: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description LLM provider setting removed */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Unauthorized */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_profile: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Current user profile with role */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UserProfileResponseDto"];
                };
            };
            /** @description Unauthorized */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    list_workflows: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Workflow definitions */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkflowDefinition"][];
                };
            };
            /** @description Unauthenticated */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    list_workflow_runs: {
        parameters: {
            query?: {
                workspace_id?: string;
                status?: string;
                type?: components["schemas"]["WorkflowType"];
                limit?: number;
                cursor?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Page of the caller's workflow runs */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RunsPageResponse"];
                };
            };
            /** @description Invalid cursor or limit */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Unauthenticated */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description workspace_id not owned by caller */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description workspace_id not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    trigger_workflow_run: {
        parameters: {
            query?: never;
            header?: {
                /** @description Forwarded to intelligence to dedupe retries */
                "Idempotency-Key"?: string | null;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["TriggerWorkflowRunRequestDto"];
            };
        };
        responses: {
            /** @description Workflow run created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkflowRun"];
                };
            };
            /** @description Unknown workflow_id */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Unauthenticated */
            401: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Workspace not owned by caller */
            403: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Workspace not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Invalid body (including unknown fields such as user_id) */
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence error or contract mismatch */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence unavailable */
            503: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Workflow run snapshot */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkflowRun"];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    list_workflow_run_agents: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Agent executions in this run */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AgentExecution"][];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_agent: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
                execution_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Agent execution details */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AgentExecution"];
                };
            };
            /** @description Unknown run/execution or run not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    list_workflow_run_agent_tools: {
        parameters: {
            query?: {
                cursor?: string;
                limit?: number;
            };
            header?: never;
            path: {
                run_id: string;
                execution_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Tool invocations for this agent execution */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ToolInvocationsPageResponse"];
                };
            };
            /** @description Unknown run/execution or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_artifacts: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Artifacts produced by this run */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Artifact"][];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    cancel_workflow_run: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Run after cancellation (idempotent) */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkflowRun"];
                };
            };
            /** @description Unknown run or not owned by caller; nothing is forwarded upstream */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_delegations: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Delegations in this run */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Delegation"][];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_delegation_graph: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Agent delegation graph */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DelegationGraphResponse"];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    workflow_run_events: {
        parameters: {
            query?: {
                after_seq?: number;
            };
            header?: {
                /** @description Resume cursor, used only when after_seq is absent */
                "Last-Event-ID"?: string | null;
            };
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description text/event-stream of ExecutionEvent frames (id: seq, event: type, data: ExecutionEvent JSON) */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "text/event-stream": unknown;
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_events_history: {
        parameters: {
            query?: {
                after_seq?: number;
                limit?: number;
            };
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Historical execution events */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["EventsHistoryPageResponse"];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_graph: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Task dependency graph */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TaskGraphResponse"];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_logs: {
        parameters: {
            query?: {
                cursor?: string;
                limit?: number;
                severity?: string;
                task_id?: string;
                execution_id?: string;
            };
            header?: never;
            path: {
                run_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Page of execution logs */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LogsPageResponse"];
                };
            };
            /** @description Unknown run or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_task: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                run_id: string;
                task_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Task details */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkflowTask"];
                };
            };
            /** @description Unknown run/task or run not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workflow_run_tool_output: {
        parameters: {
            query?: {
                offset?: number;
                limit?: number;
            };
            header?: never;
            path: {
                run_id: string;
                tool_invocation_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Chunked tool invocation output */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ToolOutputResponse"];
                };
            };
            /** @description Unknown run/tool or not owned by caller */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Intelligence response did not match contract */
            502: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    list_workspaces: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description List workspaces */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkspaceResponseDto"][];
                };
            };
        };
    };
    create_workspace: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateWorkspaceRequestDto"];
            };
        };
        responses: {
            /** @description Workspace created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkspaceResponseDto"];
                };
            };
            /** @description Invalid payload */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    get_workspace: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Workspace details */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WorkspaceResponseDto"];
                };
            };
            /** @description Workspace not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    toggle_learning_unit_completed: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
                lesson_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Learning unit completion toggled */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": unknown;
                };
            };
            /** @description Learning unit not found */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
}

// OpenFang Agents Page — Multi-step spawn wizard, detail view with tabs, file editor, personality presets
'use strict';

function agentsPage() {
  return {
    tab: 'agents',
    activeChatAgent: null,
    // -- Agents state --
    showSpawnModal: false,
    showDetailModal: false,
    detailAgent: null,
    spawnMode: 'wizard',
    spawning: false,
    spawnToml: '',
    filterState: 'all',
    loading: true,
    loadError: '',
    spawnForm: {
      name: '',
      provider: 'groq',
      model: 'llama-3.3-70b-versatile',
      systemPrompt: 'You are a helpful assistant.',
      profile: 'full',
      caps: { memory_read: true, memory_write: true, network: false, shell: false, agent_spawn: false }
    },

    // -- Multi-step wizard state --
    spawnStep: 1,
    spawnIdentity: { emoji: '', color: '#FF5C00', archetype: '' },
    selectedPreset: '',
    soulContent: '',
    emojiOptions: [
      '\u{1F916}', '\u{1F4BB}', '\u{1F50D}', '\u{270D}\uFE0F', '\u{1F4CA}', '\u{1F6E0}\uFE0F',
      '\u{1F4AC}', '\u{1F393}', '\u{1F310}', '\u{1F512}', '\u{26A1}', '\u{1F680}',
      '\u{1F9EA}', '\u{1F3AF}', '\u{1F4D6}', '\u{1F9D1}\u200D\u{1F4BB}', '\u{1F4E7}', '\u{1F3E2}',
      '\u{2764}\uFE0F', '\u{1F31F}', '\u{1F527}', '\u{1F4DD}', '\u{1F4A1}', '\u{1F3A8}'
    ],
    archetypeOptions: ['Assistant', 'Researcher', 'Coder', 'Writer', 'DevOps', 'Support', 'Analyst', 'Custom'],
    personalityPresets: [
      { id: 'professional', label: 'Professional', soul: 'Communicate in a clear, professional tone. Be direct and structured. Use formal language and data-driven reasoning. Prioritize accuracy over personality.' },
      { id: 'friendly', label: 'Friendly', soul: 'Be warm, approachable, and conversational. Use casual language and show genuine interest in the user. Add personality to your responses while staying helpful.' },
      { id: 'technical', label: 'Technical', soul: 'Focus on technical accuracy and depth. Use precise terminology. Show your work and reasoning. Prefer code examples and structured explanations.' },
      { id: 'creative', label: 'Creative', soul: 'Be imaginative and expressive. Use vivid language, analogies, and unexpected connections. Encourage creative thinking and explore multiple perspectives.' },
      { id: 'concise', label: 'Concise', soul: 'Be extremely brief and to the point. No filler, no pleasantries. Answer in the fewest words possible while remaining accurate and complete.' },
      { id: 'mentor', label: 'Mentor', soul: 'Be patient and encouraging like a great teacher. Break down complex topics step by step. Ask guiding questions. Celebrate progress and build confidence.' }
    ],

    // -- Detail modal tabs --
    detailTab: 'info',
    agentFiles: [],
    editingFile: null,
    fileContent: '',
    fileSaving: false,
    filesLoading: false,
    configForm: {},
    configSaving: false,
    // -- Tool filters --
    toolFilters: { tool_allowlist: [], tool_blocklist: [] },
    toolFiltersLoading: false,
    newAllowTool: '',
    newBlockTool: '',
    // -- Model switch --
    editingModel: false,
    newModelValue: '',
    modelSaving: false,

    // -- Templates state --
    tplTemplates: [],
    tplProviders: [],
    tplLoading: false,
    tplLoadError: '',
    selectedCategory: 'All',
    searchQuery: '',

    builtinTemplates: [
      {
        name: 'General Assistant',
        description: 'A versatile conversational agent that can help with everyday tasks, answer questions, and provide recommendations.',
        category: 'General',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a helpful, friendly assistant. Provide clear, accurate, and concise responses. Ask clarifying questions when needed.'
      },
      {
        name: 'Code Helper',
        description: 'A programming-focused agent that writes, reviews, and debugs code across multiple languages.',
        category: 'Development',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'coding',
        system_prompt: 'You are an expert programmer. Help users write clean, efficient code. Explain your reasoning. Follow best practices and conventions for the language being used.'
      },
      {
        name: 'Researcher',
        description: 'An analytical agent that breaks down complex topics, synthesizes information, and provides cited summaries.',
        category: 'Research',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'research',
        system_prompt: 'You are a research analyst. Break down complex topics into clear explanations. Provide structured analysis with key findings. Cite sources when available.'
      },
      {
        name: 'Writer',
        description: 'A creative writing agent that helps with drafting, editing, and improving written content of all kinds.',
        category: 'Writing',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a skilled writer and editor. Help users create polished content. Adapt your tone and style to match the intended audience. Offer constructive suggestions for improvement.'
      },
      {
        name: 'Data Analyst',
        description: 'A data-focused agent that helps analyze datasets, create queries, and interpret statistical results.',
        category: 'Development',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'coding',
        system_prompt: 'You are a data analysis expert. Help users understand their data, write SQL/Python queries, and interpret results. Present findings clearly with actionable insights.'
      },
      {
        name: 'DevOps Engineer',
        description: 'A systems-focused agent for CI/CD, infrastructure, Docker, and deployment troubleshooting.',
        category: 'Development',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'automation',
        system_prompt: 'You are a DevOps engineer. Help with CI/CD pipelines, Docker, Kubernetes, infrastructure as code, and deployment. Prioritize reliability and security.'
      },
      {
        name: 'Customer Support',
        description: 'A professional, empathetic agent for handling customer inquiries and resolving issues.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'messaging',
        system_prompt: 'You are a professional customer support representative. Be empathetic, patient, and solution-oriented. Acknowledge concerns before offering solutions. Escalate complex issues appropriately.'
      },
      {
        name: 'Tutor',
        description: 'A patient educational agent that explains concepts step-by-step and adapts to the learner\'s level.',
        category: 'General',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a patient and encouraging tutor. Explain concepts step by step, starting from fundamentals. Use analogies and examples. Check understanding before moving on. Adapt to the learner\'s pace.'
      },
      {
        name: 'API Designer',
        description: 'An agent specialized in RESTful API design, OpenAPI specs, and integration architecture.',
        category: 'Development',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'coding',
        system_prompt: 'You are an API design expert. Help users design clean, consistent RESTful APIs following best practices. Cover endpoint naming, request/response schemas, error handling, and versioning.'
      },
      {
        name: 'Meeting Notes',
        description: 'Summarizes meeting transcripts into structured notes with action items and key decisions.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'minimal',
        system_prompt: 'You are a meeting summarizer. When given a meeting transcript or notes, produce a structured summary with: key decisions, action items (with owners), discussion highlights, and follow-up questions.'
      },
      // ── Vietnamese Business Agents ──
      {
        name: 'Kế toán (Accountant)',
        description: 'Kế toán viên AI — hỗ trợ sổ sách, báo cáo tài chính, thuế GTGT/TNDN/TNCN, và BHXH theo VAS.',
        category: 'Finance',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are an expert Vietnamese accountant. Help with bookkeeping, financial statements (VAS), tax compliance (VAT, CIT, PIT), payroll, and social insurance (BHXH/BHYT/BHTN). Reference Vietnamese laws and circulars.'
      },
      {
        name: 'Nhân sự (HR Manager)',
        description: 'Quản lý nhân sự — tuyển dụng, đánh giá, chính sách lao động theo Bộ luật LĐ 2019.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese HR management expert. Help with recruitment, labor law compliance (Labor Code 2019), performance reviews, compensation design, training plans, and employee relations.'
      },
      {
        name: 'Thương mại điện tử (E-Commerce)',
        description: 'Chuyên gia TMĐT — tối ưu listing, giá, vận hành trên Shopee/Lazada/TikTok Shop.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese e-commerce operations expert for Shopee, Lazada, and TikTok Shop. Help with product listing optimization, pricing strategy, inventory management, and advertising campaigns.'
      },
      {
        name: 'Chiến lược Marketing',
        description: 'Lập kế hoạch chiến dịch, phân tích thị trường VN, và tối ưu marketing funnel.',
        category: 'Marketing',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese market digital marketing strategist. Help with campaign planning, audience targeting, Facebook/Zalo/TikTok ads optimization, content marketing, and ROI analysis.'
      },
      {
        name: 'SEO Specialist',
        description: 'Tối ưu công cụ tìm kiếm, phân tích từ khóa tiếng Việt, và chiến lược nội dung SEO.',
        category: 'Marketing',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are an SEO expert for Vietnamese websites. Help with keyword research, on-page optimization, technical SEO, local SEO (Google Business), and content strategy for Vietnamese search queries.'
      },
      {
        name: 'Cố vấn Khởi nghiệp',
        description: 'Startup advisor — Lean Canvas, gọi vốn, unit economics, và hệ sinh thái VC Việt Nam.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese startup ecosystem advisor. Help with business model design (Lean Canvas), fundraising strategy, pitch deck preparation, financial modeling, go-to-market plans, and team building.'
      },
      {
        name: 'Bất động sản (Real Estate)',
        description: 'Phân tích thị trường BĐS Việt Nam, định giá, đầu tư, và Luật Đất đai 2024.',
        category: 'Finance',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese real estate market expert. Help with property valuation, market analysis, investment modeling, rental yield calculation, and legal framework (Land Law 2024, property ownership rules).'
      },
      {
        name: 'Logistics',
        description: 'Tối ưu chuỗi cung ứng, vận chuyển GHN/GHTK/J&T, và thủ tục hải quan VN.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese logistics and supply chain expert. Help with shipping optimization, carrier selection (GHN, GHTK, J&T), warehouse management, customs procedures, and FTA benefits (EVFTA, CPTPP, RCEP).'
      },
      {
        name: 'Product Manager',
        description: 'Quản lý sản phẩm — PRD, user research, roadmap, Agile/Scrum, và data-driven decisions.',
        category: 'Development',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are an expert product manager. Help with PRDs, user research, feature prioritization (RICE/MoSCoW), roadmapping, agile processes, and data-driven product decisions.'
      },
      {
        name: 'Project Manager',
        description: 'Quản lý dự án — Gantt, RAID, Scrum, risk management, và stakeholder communication.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are an expert project manager. Help with project planning (WBS, Gantt), risk management (RAID logs), Agile/Scrum, stakeholder communication, budget tracking, and delivery management.'
      },
      {
        name: 'Thương hiệu (Brand Strategy)',
        description: 'Xây dựng brand identity, định vị, naming, và chiến lược truyền thông thương hiệu.',
        category: 'Marketing',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a brand strategist specializing in the Vietnamese market. Help with brand positioning, identity development, naming, messaging frameworks, and brand voice guidelines.'
      },
      {
        name: 'Sáng tạo Nội dung',
        description: 'Viết bài blog, video script, podcast, email marketing, và storytelling đa nền tảng.',
        category: 'Writing',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are an expert content creator. Help with blog posts, video scripts, podcast episodes, email campaigns, and storytelling. Adapt content for different platforms and Vietnamese audiences.'
      },
      {
        name: 'Pháp luật VN (Legal)',
        description: 'Tư vấn hợp đồng, Luật Doanh nghiệp, SHTT, PDPD, và pháp luật Việt Nam.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese legal information advisor. Help with contract review, business law (Enterprise Law 2020), IP registration, labor law, data privacy (PDPD Decree 13/2023), and dispute resolution. DISCLAIMER: Provide legal information, not legal advice.'
      },
      {
        name: 'Business Analyst',
        description: 'Phân tích nghiệp vụ — BPMN, requirements engineering, và tối ưu quy trình.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are an expert business analyst. Help with requirements engineering, process modeling (BPMN), gap analysis, solution assessment, and stakeholder management. Bridge business needs and technical solutions.'
      },
      {
        name: 'CX Manager',
        description: 'Quản lý trải nghiệm khách hàng — journey map, NPS/CSAT, retention strategy.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a customer experience expert. Help design customer journey maps, VoC programs (NPS/CSAT/CES), retention strategies, loyalty programs, and service design improvements.'
      },
      {
        name: 'UI/UX Designer',
        description: 'Thiết kế giao diện — wireframe specs, design systems, accessibility, và developer handoff.',
        category: 'Development',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a UI/UX design expert. Help with wireframe specifications, design system creation, accessibility (WCAG), interaction design, and developer handoff documentation.'
      },
      {
        name: 'Data Engineer',
        description: 'Data pipeline, ETL, SQL optimization, data modeling, và data warehouse architecture.',
        category: 'Development',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'coding',
        system_prompt: 'You are an expert data engineer. Help with ETL/ELT pipelines, SQL optimization, data modeling (star/snowflake schema), data quality frameworks, and cloud data infrastructure.'
      },
      {
        name: 'Gia sư (Education Tutor)',
        description: 'Hỗ trợ học tập — giải bài Toán/Lý/Hóa, luyện thi IELTS, và kế hoạch ôn tập.',
        category: 'General',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a Vietnamese education tutor. Help students with math, science, languages, and exam preparation (THPT, IELTS, TOEFL). Explain step-by-step, adapt to the student level, and create study plans.'
      },
      {
        name: 'Sức khỏe (Healthcare)',
        description: 'Thông tin y tế, dinh dưỡng, thể dục, và chăm sóc sức khỏe phòng ngừa.',
        category: 'General',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a health information advisor. Provide evidence-based health information, nutrition planning, exercise routines, and preventive care guidance. DISCLAIMER: Not medical advice — recommend consulting a healthcare professional for specific concerns.'
      },
      {
        name: 'Tuân thủ (Compliance)',
        description: 'Quản lý compliance, AML, data privacy PDPD, và kiểm soát rủi ro doanh nghiệp.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a regulatory compliance expert for Vietnamese businesses. Help with AML programs, data privacy (PDPD), internal controls, audit preparation, anti-corruption policies, and risk management.'
      },
      {
        name: 'Mua hàng (Procurement)',
        description: 'Tìm nguồn cung, đàm phán nhà cung cấp, RFP/RFQ, và tối ưu chi phí mua sắm.',
        category: 'Business',
        provider: 'groq',
        model: 'llama-3.3-70b-versatile',
        profile: 'full',
        system_prompt: 'You are a strategic procurement expert. Help with supplier evaluation (RFP/RFQ), negotiation strategies, cost optimization (TCO analysis), contract management, and purchase order processes.'
      }
    ],

    // ── Profile Descriptions ──
    profileDescriptions: {
      minimal: { label: 'Minimal', desc: 'Read-only file access' },
      coding: { label: 'Coding', desc: 'Files + shell + web fetch' },
      research: { label: 'Research', desc: 'Web search + file read/write' },
      messaging: { label: 'Messaging', desc: 'Agents + memory access' },
      automation: { label: 'Automation', desc: 'All tools except custom' },
      balanced: { label: 'Balanced', desc: 'General-purpose tool set' },
      precise: { label: 'Precise', desc: 'Focused tool set for accuracy' },
      creative: { label: 'Creative', desc: 'Full tools with creative emphasis' },
      full: { label: 'Full', desc: 'All 35+ tools' }
    },
    profileInfo: function(name) {
      return this.profileDescriptions[name] || { label: name, desc: '' };
    },

    // ── Tool Preview in Spawn Modal ──
    spawnProfiles: [],
    spawnProfilesLoaded: false,
    async loadSpawnProfiles() {
      if (this.spawnProfilesLoaded) return;
      try {
        var data = await OpenFangAPI.get('/api/profiles');
        this.spawnProfiles = data.profiles || [];
        this.spawnProfilesLoaded = true;
      } catch(e) { this.spawnProfiles = []; }
    },
    get selectedProfileTools() {
      var pname = this.spawnForm.profile;
      var match = this.spawnProfiles.find(function(p) { return p.name === pname; });
      if (match && match.tools) return match.tools.slice(0, 15);
      return [];
    },

    get agents() { return Alpine.store('app').agents; },

    get filteredAgents() {
      var f = this.filterState;
      if (f === 'all') return this.agents;
      return this.agents.filter(function(a) { return a.state.toLowerCase() === f; });
    },

    get runningCount() {
      return this.agents.filter(function(a) { return a.state === 'Running'; }).length;
    },

    get stoppedCount() {
      return this.agents.filter(function(a) { return a.state !== 'Running'; }).length;
    },

    // -- Templates computed --
    get categories() {
      var cats = { 'All': true };
      this.builtinTemplates.forEach(function(t) { cats[t.category] = true; });
      this.tplTemplates.forEach(function(t) { if (t.category) cats[t.category] = true; });
      return Object.keys(cats);
    },

    get filteredBuiltins() {
      var self = this;
      return this.builtinTemplates.filter(function(t) {
        if (self.selectedCategory !== 'All' && t.category !== self.selectedCategory) return false;
        if (self.searchQuery) {
          var q = self.searchQuery.toLowerCase();
          if (t.name.toLowerCase().indexOf(q) === -1 &&
              t.description.toLowerCase().indexOf(q) === -1) return false;
        }
        return true;
      });
    },

    get filteredCustom() {
      var self = this;
      return this.tplTemplates.filter(function(t) {
        if (self.searchQuery) {
          var q = self.searchQuery.toLowerCase();
          if ((t.name || '').toLowerCase().indexOf(q) === -1 &&
              (t.description || '').toLowerCase().indexOf(q) === -1) return false;
        }
        return true;
      });
    },

    isProviderConfigured(providerName) {
      if (!providerName) return false;
      var p = this.tplProviders.find(function(pr) { return pr.id === providerName; });
      return p ? p.auth_status === 'configured' : false;
    },

    async init() {
      var self = this;
      this.loading = true;
      this.loadError = '';
      try {
        await Alpine.store('app').refreshAgents();
      } catch(e) {
        this.loadError = e.message || 'Could not load agents. Is the daemon running?';
      }
      this.loading = false;

      // If a pending agent was set (e.g. from wizard or redirect), open chat inline
      var store = Alpine.store('app');
      if (store.pendingAgent) {
        this.activeChatAgent = store.pendingAgent;
      }
      // Watch for future pendingAgent changes
      this.$watch('$store.app.pendingAgent', function(agent) {
        if (agent) {
          self.activeChatAgent = agent;
        }
      });
    },

    async loadData() {
      this.loading = true;
      this.loadError = '';
      try {
        await Alpine.store('app').refreshAgents();
      } catch(e) {
        this.loadError = e.message || 'Could not load agents.';
      }
      this.loading = false;
    },

    async loadTemplates() {
      this.tplLoading = true;
      this.tplLoadError = '';
      try {
        var results = await Promise.all([
          OpenFangAPI.get('/api/templates'),
          OpenFangAPI.get('/api/providers').catch(function() { return { providers: [] }; })
        ]);
        this.tplTemplates = results[0].templates || [];
        this.tplProviders = results[1].providers || [];
      } catch(e) {
        this.tplTemplates = [];
        this.tplLoadError = e.message || 'Could not load templates.';
      }
      this.tplLoading = false;
    },

    chatWithAgent(agent) {
      Alpine.store('app').pendingAgent = agent;
      this.activeChatAgent = agent;
    },

    closeChat() {
      this.activeChatAgent = null;
      OpenFangAPI.wsDisconnect();
    },

    showDetail(agent) {
      this.detailAgent = agent;
      this.detailTab = 'info';
      this.agentFiles = [];
      this.editingFile = null;
      this.fileContent = '';
      this.configForm = {
        name: agent.name || '',
        system_prompt: agent.system_prompt || '',
        emoji: (agent.identity && agent.identity.emoji) || '',
        color: (agent.identity && agent.identity.color) || '#FF5C00',
        archetype: (agent.identity && agent.identity.archetype) || '',
        vibe: (agent.identity && agent.identity.vibe) || ''
      };
      this.showDetailModal = true;
    },

    killAgent(agent) {
      var self = this;
      OpenFangToast.confirm('Stop Agent', 'Stop agent "' + agent.name + '"? The agent will be shut down.', async function() {
        try {
          await OpenFangAPI.del('/api/agents/' + agent.id);
          OpenFangToast.success('Agent "' + agent.name + '" stopped');
          self.showDetailModal = false;
          await Alpine.store('app').refreshAgents();
        } catch(e) {
          OpenFangToast.error('Failed to stop agent: ' + e.message);
        }
      });
    },

    killAllAgents() {
      var list = this.filteredAgents;
      if (!list.length) return;
      OpenFangToast.confirm('Stop All Agents', 'Stop ' + list.length + ' agent(s)? All agents will be shut down.', async function() {
        var errors = [];
        for (var i = 0; i < list.length; i++) {
          try {
            await OpenFangAPI.del('/api/agents/' + list[i].id);
          } catch(e) { errors.push(list[i].name + ': ' + e.message); }
        }
        await Alpine.store('app').refreshAgents();
        if (errors.length) {
          OpenFangToast.error('Some agents failed to stop: ' + errors.join(', '));
        } else {
          OpenFangToast.success(list.length + ' agent(s) stopped');
        }
      });
    },

    // ── Multi-step wizard navigation ──
    openSpawnWizard() {
      this.showSpawnModal = true;
      this.spawnStep = 1;
      this.spawnMode = 'wizard';
      this.spawnIdentity = { emoji: '', color: '#FF5C00', archetype: '' };
      this.selectedPreset = '';
      this.soulContent = '';
      this.spawnForm.name = '';
      this.spawnForm.systemPrompt = 'You are a helpful assistant.';
      this.spawnForm.profile = 'full';
    },

    nextStep() {
      if (this.spawnStep === 1 && !this.spawnForm.name.trim()) {
        OpenFangToast.warn('Please enter an agent name');
        return;
      }
      if (this.spawnStep < 5) this.spawnStep++;
    },

    prevStep() {
      if (this.spawnStep > 1) this.spawnStep--;
    },

    selectPreset(preset) {
      this.selectedPreset = preset.id;
      this.soulContent = preset.soul;
    },

    generateToml() {
      var f = this.spawnForm;
      var si = this.spawnIdentity;
      var lines = [
        'name = "' + f.name + '"',
        'module = "builtin:chat"'
      ];
      if (f.profile && f.profile !== 'custom') {
        lines.push('profile = "' + f.profile + '"');
      }
      lines.push('', '[model]');
      lines.push('provider = "' + f.provider + '"');
      lines.push('model = "' + f.model + '"');
      lines.push('system_prompt = "' + f.systemPrompt.replace(/"/g, '\\"') + '"');
      if (f.profile === 'custom') {
        lines.push('', '[capabilities]');
        if (f.caps.memory_read) lines.push('memory_read = ["*"]');
        if (f.caps.memory_write) lines.push('memory_write = ["self.*"]');
        if (f.caps.network) lines.push('network = ["*"]');
        if (f.caps.shell) lines.push('shell = ["*"]');
        if (f.caps.agent_spawn) lines.push('agent_spawn = true');
      }
      return lines.join('\n');
    },

    async setMode(agent, mode) {
      try {
        await OpenFangAPI.put('/api/agents/' + agent.id + '/mode', { mode: mode });
        agent.mode = mode;
        OpenFangToast.success('Mode set to ' + mode);
        await Alpine.store('app').refreshAgents();
      } catch(e) {
        OpenFangToast.error('Failed to set mode: ' + e.message);
      }
    },

    async spawnAgent() {
      this.spawning = true;
      var toml = this.spawnMode === 'wizard' ? this.generateToml() : this.spawnToml;
      if (!toml.trim()) {
        this.spawning = false;
        OpenFangToast.warn('Manifest is empty \u2014 enter agent config first');
        return;
      }

      try {
        var res = await OpenFangAPI.post('/api/agents', { manifest_toml: toml });
        if (res.agent_id) {
          // Post-spawn: update identity + write SOUL.md if personality preset selected
          var patchBody = {};
          if (this.spawnIdentity.emoji) patchBody.emoji = this.spawnIdentity.emoji;
          if (this.spawnIdentity.color) patchBody.color = this.spawnIdentity.color;
          if (this.spawnIdentity.archetype) patchBody.archetype = this.spawnIdentity.archetype;
          if (this.selectedPreset) patchBody.vibe = this.selectedPreset;

          if (Object.keys(patchBody).length) {
            OpenFangAPI.patch('/api/agents/' + res.agent_id + '/config', patchBody).catch(function(e) { console.warn('Post-spawn config patch failed:', e.message); });
          }
          if (this.soulContent.trim()) {
            OpenFangAPI.put('/api/agents/' + res.agent_id + '/files/SOUL.md', { content: '# Soul\n' + this.soulContent }).catch(function(e) { console.warn('SOUL.md write failed:', e.message); });
          }

          this.showSpawnModal = false;
          this.spawnForm.name = '';
          this.spawnToml = '';
          this.spawnStep = 1;
          OpenFangToast.success('Agent "' + (res.name || 'new') + '" spawned');
          await Alpine.store('app').refreshAgents();
          this.chatWithAgent({ id: res.agent_id, name: res.name, model_provider: '?', model_name: '?' });
        } else {
          OpenFangToast.error('Spawn failed: ' + (res.error || 'Unknown error'));
        }
      } catch(e) {
        OpenFangToast.error('Failed to spawn agent: ' + e.message);
      }
      this.spawning = false;
    },

    // ── Detail modal: Files tab ──
    async loadAgentFiles() {
      if (!this.detailAgent) return;
      this.filesLoading = true;
      try {
        var data = await OpenFangAPI.get('/api/agents/' + this.detailAgent.id + '/files');
        this.agentFiles = data.files || [];
      } catch(e) {
        this.agentFiles = [];
        OpenFangToast.error('Failed to load files: ' + e.message);
      }
      this.filesLoading = false;
    },

    async openFile(file) {
      if (!file.exists) {
        // Create with empty content
        this.editingFile = file.name;
        this.fileContent = '';
        return;
      }
      try {
        var data = await OpenFangAPI.get('/api/agents/' + this.detailAgent.id + '/files/' + encodeURIComponent(file.name));
        this.editingFile = file.name;
        this.fileContent = data.content || '';
      } catch(e) {
        OpenFangToast.error('Failed to read file: ' + e.message);
      }
    },

    async saveFile() {
      if (!this.editingFile || !this.detailAgent) return;
      this.fileSaving = true;
      try {
        await OpenFangAPI.put('/api/agents/' + this.detailAgent.id + '/files/' + encodeURIComponent(this.editingFile), { content: this.fileContent });
        OpenFangToast.success(this.editingFile + ' saved');
        await this.loadAgentFiles();
      } catch(e) {
        OpenFangToast.error('Failed to save file: ' + e.message);
      }
      this.fileSaving = false;
    },

    closeFileEditor() {
      this.editingFile = null;
      this.fileContent = '';
    },

    // ── Detail modal: Config tab ──
    async saveConfig() {
      if (!this.detailAgent) return;
      this.configSaving = true;
      try {
        await OpenFangAPI.patch('/api/agents/' + this.detailAgent.id + '/config', this.configForm);
        OpenFangToast.success('Config updated');
        await Alpine.store('app').refreshAgents();
      } catch(e) {
        OpenFangToast.error('Failed to save config: ' + e.message);
      }
      this.configSaving = false;
    },

    // ── Clone agent ──
    async cloneAgent(agent) {
      var newName = (agent.name || 'agent') + '-copy';
      try {
        var res = await OpenFangAPI.post('/api/agents/' + agent.id + '/clone', { new_name: newName });
        if (res.agent_id) {
          OpenFangToast.success('Cloned as "' + res.name + '"');
          await Alpine.store('app').refreshAgents();
          this.showDetailModal = false;
        }
      } catch(e) {
        OpenFangToast.error('Clone failed: ' + e.message);
      }
    },

    // -- Template methods --
    async spawnFromTemplate(name) {
      try {
        var data = await OpenFangAPI.get('/api/templates/' + encodeURIComponent(name));
        if (data.manifest_toml) {
          var res = await OpenFangAPI.post('/api/agents', { manifest_toml: data.manifest_toml });
          if (res.agent_id) {
            OpenFangToast.success('Agent "' + (res.name || name) + '" spawned from template');
            await Alpine.store('app').refreshAgents();
            this.chatWithAgent({ id: res.agent_id, name: res.name || name, model_provider: '?', model_name: '?' });
          }
        }
      } catch(e) {
        OpenFangToast.error('Failed to spawn from template: ' + e.message);
      }
    },

    // ── Clear agent history ──
    async clearHistory(agent) {
      var self = this;
      OpenFangToast.confirm('Clear History', 'Clear all conversation history for "' + agent.name + '"? This cannot be undone.', async function() {
        try {
          await OpenFangAPI.del('/api/agents/' + agent.id + '/history');
          OpenFangToast.success('History cleared for "' + agent.name + '"');
        } catch(e) {
          OpenFangToast.error('Failed to clear history: ' + e.message);
        }
      });
    },

    // ── Model switch ──
    async changeModel() {
      if (!this.detailAgent || !this.newModelValue.trim()) return;
      this.modelSaving = true;
      try {
        await OpenFangAPI.put('/api/agents/' + this.detailAgent.id + '/model', { model: this.newModelValue.trim() });
        OpenFangToast.success('Model changed (memory reset)');
        this.editingModel = false;
        await Alpine.store('app').refreshAgents();
        // Refresh detailAgent
        var agents = Alpine.store('app').agents;
        for (var i = 0; i < agents.length; i++) {
          if (agents[i].id === this.detailAgent.id) { this.detailAgent = agents[i]; break; }
        }
      } catch(e) {
        OpenFangToast.error('Failed to change model: ' + e.message);
      }
      this.modelSaving = false;
    },

    // ── Tool filters ──
    async loadToolFilters() {
      if (!this.detailAgent) return;
      this.toolFiltersLoading = true;
      try {
        this.toolFilters = await OpenFangAPI.get('/api/agents/' + this.detailAgent.id + '/tools');
      } catch(e) {
        this.toolFilters = { tool_allowlist: [], tool_blocklist: [] };
      }
      this.toolFiltersLoading = false;
    },

    addAllowTool() {
      var t = this.newAllowTool.trim();
      if (t && this.toolFilters.tool_allowlist.indexOf(t) === -1) {
        this.toolFilters.tool_allowlist.push(t);
        this.newAllowTool = '';
        this.saveToolFilters();
      }
    },

    removeAllowTool(tool) {
      this.toolFilters.tool_allowlist = this.toolFilters.tool_allowlist.filter(function(t) { return t !== tool; });
      this.saveToolFilters();
    },

    addBlockTool() {
      var t = this.newBlockTool.trim();
      if (t && this.toolFilters.tool_blocklist.indexOf(t) === -1) {
        this.toolFilters.tool_blocklist.push(t);
        this.newBlockTool = '';
        this.saveToolFilters();
      }
    },

    removeBlockTool(tool) {
      this.toolFilters.tool_blocklist = this.toolFilters.tool_blocklist.filter(function(t) { return t !== tool; });
      this.saveToolFilters();
    },

    async saveToolFilters() {
      if (!this.detailAgent) return;
      try {
        await OpenFangAPI.put('/api/agents/' + this.detailAgent.id + '/tools', this.toolFilters);
      } catch(e) {
        OpenFangToast.error('Failed to update tool filters: ' + e.message);
      }
    },

    async spawnBuiltin(t) {
      var toml = 'name = "' + t.name + '"\n';
      toml += 'description = "' + t.description.replace(/"/g, '\\"') + '"\n';
      toml += 'module = "builtin:chat"\n';
      toml += 'profile = "' + t.profile + '"\n\n';
      toml += '[model]\nprovider = "' + t.provider + '"\nmodel = "' + t.model + '"\n';
      toml += 'system_prompt = """\n' + t.system_prompt + '\n"""\n';

      try {
        var res = await OpenFangAPI.post('/api/agents', { manifest_toml: toml });
        if (res.agent_id) {
          OpenFangToast.success('Agent "' + t.name + '" spawned');
          await Alpine.store('app').refreshAgents();
          this.chatWithAgent({ id: res.agent_id, name: t.name, model_provider: t.provider, model_name: t.model });
        }
      } catch(e) {
        OpenFangToast.error('Failed to spawn agent: ' + e.message);
      }
    }
  };
}

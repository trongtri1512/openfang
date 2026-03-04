// OpenFang Tenants Page — Multi-tenant management with 7-tab detail view
'use strict';

function tenantsPage() {
    return {
        view: 'list', // 'list' or 'detail'
        tenants: [],
        total: 0,
        running: 0,
        loading: true,
        loadError: '',
        searchQuery: '',
        statusFilter: 'all',
        // Detail view
        currentTenant: null,
        detailTab: 'overview',
        tenantStats: null,
        tenantLogs: [],
        statsLoading: false,
        // Create modal
        showCreateModal: false,
        creating: false,
        createForm: { name: '', plan: 'free', provider: 'groq', model: 'llama-3.3-70b-versatile', temperature: 0.7 },
        // Members
        newMemberEmail: '',
        addingMember: false,
        // Config tab
        showApiKey: false,
        testingApiKey: false,
        apiKeyTestResult: '',
        // Config.toml tab
        configToml: '',
        configTomlLoading: false,
        // Channels tab
        tenantChannels: [],
        channelsLoading: false,
        showAddChannelModal: false,
        addChannelForm: { channel_type: 'telegram', name: '' },
        // Usage tab
        usageData: null,
        usageLoading: false,
        // Assistant tab
        assistantMessages: [],
        assistantInput: '',
        assistantSending: false,

        // Available providers and models
        providers: [
            { value: 'groq', label: 'Groq (Free)' },
            { value: 'openai', label: 'OpenAI' },
            { value: 'anthropic', label: 'Anthropic' },
            { value: 'ollama', label: 'Ollama (Local)' },
            { value: 'openrouter', label: 'OpenRouter' },
            { value: 'google', label: 'Google AI' },
        ],
        providerModels: {
            groq: ['llama-3.3-70b-versatile', 'llama-3.1-8b-instant', 'mixtral-8x7b-32768', 'gemma2-9b-it'],
            openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'],
            anthropic: ['claude-3-5-sonnet-20241022', 'claude-3-haiku-20240307', 'claude-3-opus-20240229'],
            ollama: ['qwen2.5:14b', 'llama3.2:3b', 'mistral:7b', 'codellama:13b'],
            openrouter: ['google/gemini-2.0-flash-exp:free', 'deepseek/deepseek-r1:free', 'meta-llama/llama-3.3-70b-instruct:free'],
            google: ['gemini-2.0-flash', 'gemini-1.5-pro', 'gemini-1.5-flash'],
        },
        channelTypes: [
            { value: 'telegram', label: 'Telegram', icon: '✈️' },
            { value: 'zalo', label: 'Zalo', icon: '💬' },
            { value: 'slack', label: 'Slack', icon: '💼' },
            { value: 'discord', label: 'Discord', icon: '🎮' },
            { value: 'whatsapp', label: 'WhatsApp', icon: '📱' },
            { value: 'email', label: 'Email', icon: '📧' },
            { value: 'messenger', label: 'Messenger', icon: '💬' },
            { value: 'teams', label: 'Microsoft Teams', icon: '🏢' },
            { value: 'webchat', label: 'WebChat', icon: '🌐' },
            { value: 'line', label: 'LINE', icon: '🟢' },
        ],

        async init() {
            await this.loadTenants();
        },

        async loadTenants() {
            this.loading = true;
            this.loadError = '';
            try {
                var params = new URLSearchParams();
                if (this.searchQuery.trim()) params.set('search', this.searchQuery.trim());
                if (this.statusFilter !== 'all') params.set('status', this.statusFilter);
                var url = '/api/tenants' + (params.toString() ? '?' + params.toString() : '');
                var data = await OpenFangAPI.get(url);
                this.tenants = data.tenants || [];
                this.total = data.total || 0;
                this.running = data.running || 0;
            } catch (e) {
                this.loadError = e.message || 'Could not load tenants.';
                this.tenants = [];
            }
            this.loading = false;
        },

        get filteredTenants() {
            return this.tenants;
        },

        statusColor(status) {
            if (status === 'running') return 'var(--success)';
            if (status === 'stopped') return 'var(--text-dim)';
            return 'var(--danger)';
        },

        planColor(plan) {
            if (plan === 'pro') return 'var(--accent)';
            if (plan === 'enterprise') return 'var(--info)';
            return 'var(--text-dim)';
        },

        planLabel(plan) {
            if (plan === 'pro') return 'Pro';
            if (plan === 'enterprise') return 'Enterprise';
            return 'Free';
        },

        formatDate(iso) {
            if (!iso) return '-';
            var d = new Date(iso);
            var months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
            return months[d.getMonth()] + ' ' + d.getDate() + ', ' + d.getFullYear();
        },

        get currentModels() {
            if (!this.currentTenant) return [];
            return this.providerModels[this.currentTenant.provider] || [];
        },

        get maskedApiKey() {
            if (!this.currentTenant || !this.currentTenant.api_key) return '';
            var k = this.currentTenant.api_key;
            if (k.length <= 8) return '••••••••';
            return '•••••••' + k.slice(-4);
        },

        // ── Create ──
        async createTenant() {
            if (!this.createForm.name.trim()) {
                OpenFangToast.warn('Name is required');
                return;
            }
            this.creating = true;
            try {
                await OpenFangAPI.post('/api/tenants', this.createForm);
                OpenFangToast.success('Tenant created');
                this.showCreateModal = false;
                this.createForm = { name: '', plan: 'free', provider: 'groq', model: 'llama-3.3-70b-versatile', temperature: 0.7 };
                await this.loadTenants();
            } catch (e) {
                OpenFangToast.error('Failed: ' + e.message);
            }
            this.creating = false;
        },

        // ── Detail view ──
        async openDetail(tenant) {
            this.currentTenant = tenant;
            this.detailTab = 'overview';
            this.view = 'detail';
            await this.loadStats();
        },

        backToList() {
            this.view = 'list';
            this.currentTenant = null;
            this.loadTenants();
        },

        openDashboard() {
            location.hash = 'agents';
        },

        async loadStats() {
            if (!this.currentTenant) return;
            this.statsLoading = true;
            try {
                this.tenantStats = await OpenFangAPI.get('/api/tenants/' + this.currentTenant.id + '/stats');
            } catch (e) {
                this.tenantStats = null;
            }
            this.statsLoading = false;
        },

        async loadLogs() {
            if (!this.currentTenant) return;
            try {
                var data = await OpenFangAPI.get('/api/tenants/' + this.currentTenant.id + '/logs');
                this.tenantLogs = data.logs || [];
            } catch (e) {
                this.tenantLogs = [];
            }
        },

        // ── Tab switch handler ──
        async switchTab(tab) {
            this.detailTab = tab;
            if (tab === 'members') await this.refreshCurrentTenant();
            if (tab === 'config.toml') await this.loadConfigToml();
            if (tab === 'channels') await this.loadTenantChannels();
            if (tab === 'usage') await this.loadUsage();
        },

        // ── Actions ──
        async restartTenant() {
            if (!this.currentTenant) return;
            try {
                await OpenFangAPI.post('/api/tenants/' + this.currentTenant.id + '/restart');
                this.currentTenant.status = 'running';
                OpenFangToast.success('Tenant restarted');
                await this.refreshCurrentTenant();
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        async stopTenant() {
            if (!this.currentTenant) return;
            try {
                await OpenFangAPI.post('/api/tenants/' + this.currentTenant.id + '/stop');
                this.currentTenant.status = 'stopped';
                OpenFangToast.success('Tenant stopped');
                await this.refreshCurrentTenant();
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        async deleteTenant(tenant) {
            var t = tenant || this.currentTenant;
            if (!t) return;
            var self = this;
            OpenFangToast.confirm('Delete Tenant', 'Delete "' + t.name + '"? This cannot be undone.', async function () {
                try {
                    await OpenFangAPI.del('/api/tenants/' + t.id);
                    OpenFangToast.success('Tenant deleted');
                    if (self.view === 'detail') self.backToList();
                    else await self.loadTenants();
                } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
            });
        },

        async refreshCurrentTenant() {
            if (!this.currentTenant) return;
            try {
                this.currentTenant = await OpenFangAPI.get('/api/tenants/' + this.currentTenant.id);
                await this.loadStats();
            } catch (e) { }
        },

        // ── Access Link ──
        get accessLink() {
            if (!this.currentTenant) return '';
            return location.origin + '/access/?t=' + this.currentTenant.access_token;
        },

        copyAccessLink() {
            navigator.clipboard.writeText(this.accessLink);
            OpenFangToast.success('Link copied!');
        },

        async regenerateLink() {
            if (!this.currentTenant) return;
            try {
                var data = await OpenFangAPI.post('/api/tenants/' + this.currentTenant.id + '/access-link');
                this.currentTenant.access_token = data.access_token;
                OpenFangToast.success('Link regenerated');
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        // ── Members ──
        async addMember() {
            if (!this.currentTenant || !this.newMemberEmail.trim()) return;
            this.addingMember = true;
            try {
                var data = await OpenFangAPI.post('/api/tenants/' + this.currentTenant.id + '/members', { email: this.newMemberEmail.trim() });
                this.currentTenant.members = data.members;
                this.newMemberEmail = '';
                OpenFangToast.success('Member added');
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
            this.addingMember = false;
        },

        async removeMember(email) {
            if (!this.currentTenant) return;
            try {
                var data = await OpenFangAPI.del('/api/tenants/' + this.currentTenant.id + '/members/' + encodeURIComponent(email));
                this.currentTenant.members = data.members;
                OpenFangToast.success('Member removed');
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        // ── Config edit (enhanced) ──
        async saveTenantConfig() {
            if (!this.currentTenant) return;
            try {
                this.currentTenant = await OpenFangAPI.put('/api/tenants/' + this.currentTenant.id, {
                    name: this.currentTenant.name,
                    provider: this.currentTenant.provider,
                    model: this.currentTenant.model,
                    temperature: this.currentTenant.temperature,
                    plan: this.currentTenant.plan,
                });
                OpenFangToast.success('Config saved & restarting...');
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        async testApiKey() {
            this.testingApiKey = true;
            this.apiKeyTestResult = '';
            try {
                // Simple check: just verify non-empty
                if (this.currentTenant.api_key && this.currentTenant.api_key.trim()) {
                    this.apiKeyTestResult = 'valid';
                    OpenFangToast.success('API key format looks valid');
                } else {
                    this.apiKeyTestResult = 'empty';
                    OpenFangToast.warn('No API key set — using Quick Start mode');
                }
            } catch (e) {
                this.apiKeyTestResult = 'error';
            }
            this.testingApiKey = false;
        },

        // ── Config.toml ──
        async loadConfigToml() {
            if (!this.currentTenant) return;
            this.configTomlLoading = true;
            try {
                var data = await OpenFangAPI.get('/api/tenants/' + this.currentTenant.id + '/config-toml');
                this.configToml = data.toml || '';
            } catch (e) {
                this.configToml = '# Error loading config';
            }
            this.configTomlLoading = false;
        },

        copyConfigToml() {
            navigator.clipboard.writeText(this.configToml);
            OpenFangToast.success('Config copied!');
        },

        downloadConfigToml() {
            var blob = new Blob([this.configToml], { type: 'text/plain' });
            var url = URL.createObjectURL(blob);
            var a = document.createElement('a');
            a.href = url;
            a.download = (this.currentTenant ? this.currentTenant.slug : 'tenant') + '.toml';
            a.click();
            URL.revokeObjectURL(url);
        },

        // ── Channels ──
        async loadTenantChannels() {
            if (!this.currentTenant) return;
            this.channelsLoading = true;
            try {
                var data = await OpenFangAPI.get('/api/tenants/' + this.currentTenant.id + '/channels');
                this.tenantChannels = data.channels || [];
            } catch (e) {
                this.tenantChannels = [];
            }
            this.channelsLoading = false;
        },

        async addChannel() {
            if (!this.currentTenant) return;
            try {
                var req = {
                    channel_type: this.addChannelForm.channel_type,
                    name: this.addChannelForm.name || this.addChannelForm.channel_type,
                    config: {},
                };
                var data = await OpenFangAPI.post('/api/tenants/' + this.currentTenant.id + '/channels', req);
                this.tenantChannels = data.channels || [];
                this.showAddChannelModal = false;
                this.addChannelForm = { channel_type: 'telegram', name: '' };
                OpenFangToast.success('Channel added');
                await this.refreshCurrentTenant();
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        async toggleChannel(ch) {
            if (!this.currentTenant) return;
            try {
                var data = await OpenFangAPI.put('/api/tenants/' + this.currentTenant.id + '/channels/' + encodeURIComponent(ch.channel_type), { enabled: !ch.enabled });
                this.tenantChannels = data.channels || [];
                OpenFangToast.success(ch.enabled ? 'Channel disabled' : 'Channel enabled');
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        async removeChannel(ch) {
            if (!this.currentTenant) return;
            try {
                var data = await OpenFangAPI.del('/api/tenants/' + this.currentTenant.id + '/channels/' + encodeURIComponent(ch.channel_type));
                this.tenantChannels = data.channels || [];
                OpenFangToast.success('Channel removed');
                await this.refreshCurrentTenant();
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

        channelIcon(type) {
            var found = this.channelTypes.find(function (c) { return c.value === type; });
            return found ? found.icon : '📡';
        },

        channelLabel(type) {
            var found = this.channelTypes.find(function (c) { return c.value === type; });
            return found ? found.label : type;
        },

        // ── Usage ──
        async loadUsage() {
            if (!this.currentTenant) return;
            this.usageLoading = true;
            try {
                this.usageData = await OpenFangAPI.get('/api/tenants/' + this.currentTenant.id + '/usage');
            } catch (e) {
                this.usageData = null;
            }
            this.usageLoading = false;
        },

        formatUptime(secs) {
            if (!secs) return '-';
            var d = Math.floor(secs / 86400);
            var h = Math.floor((secs % 86400) / 3600);
            var m = Math.floor((secs % 3600) / 60);
            if (d > 0) return d + 'd ' + h + 'h';
            if (h > 0) return h + 'h ' + m + 'm';
            return m + 'm';
        },

        // ── Assistant (Chat) ──
        async sendAssistantMessage() {
            if (!this.assistantInput.trim() || this.assistantSending) return;
            var msg = this.assistantInput.trim();
            this.assistantMessages.push({ role: 'user', content: msg, time: new Date().toLocaleTimeString() });
            this.assistantInput = '';
            this.assistantSending = true;
            try {
                var resp = await OpenFangAPI.post('/v1/chat/completions', {
                    model: this.currentTenant ? this.currentTenant.model : 'default',
                    messages: [{ role: 'user', content: msg }],
                    stream: false,
                });
                var reply = resp.choices && resp.choices[0] && resp.choices[0].message ? resp.choices[0].message.content : 'No response';
                this.assistantMessages.push({ role: 'assistant', content: reply, time: new Date().toLocaleTimeString() });
            } catch (e) {
                this.assistantMessages.push({ role: 'assistant', content: '⚠️ Error: ' + (e.message || 'Failed'), time: new Date().toLocaleTimeString() });
            }
            this.assistantSending = false;
        },

        // ── Helpers ──
        quotaPercent() {
            if (!this.currentTenant || !this.tenantStats) return 0;
            var limit = this.tenantStats.messages_limit;
            if (limit >= 4294967295) return 0;
            return Math.min(100, Math.round((this.tenantStats.messages_today / limit) * 100));
        },

        memoryPercent() {
            if (!this.tenantStats) return 0;
            return Math.round((this.tenantStats.memory_used_mb / this.tenantStats.memory_total_mb) * 100);
        }
    };
}

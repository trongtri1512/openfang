// OpenFang Tenants Page — Multi-tenant management with list + detail views
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

        // ── Config edit ──
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
                OpenFangToast.success('Config saved');
            } catch (e) { OpenFangToast.error('Failed: ' + e.message); }
        },

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

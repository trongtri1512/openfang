//! Portal HTML/CSS/JS  the complete single-page application.

pub const PORTAL_HTML: &str = r##"<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>OpenFang Portal</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{--o:#FF5C00;--oh:#e65200;--obg:rgba(255,92,0,.08);--ol:#fff7ed;--bg:#fff;--bg2:#f9fafb;--bg3:#f3f4f6;--t:#111827;--d:#6b7280;--m:#9ca3af;--b:#e5e7eb;--g:#22c55e;--gb:#f0fdf4;--gt:#15803d;--r:#ef4444;--rb:#fef2f2;--rt:#dc2626;--pb:#faf5ff;--pt:#7c3aed;--bb:#eff6ff;--bt:#2563eb}
body{font-family:'Inter',system-ui,sans-serif;margin:0;min-height:100vh;background:var(--bg2)}
/* Login Screen */
.login-screen{display:flex;min-height:100vh;background:var(--bg)}
.login-left{flex:1;background:var(--bg2);position:relative;display:flex;flex-direction:column;justify-content:center;padding:48px 64px;overflow:hidden}
.login-left::before{content:'';position:absolute;inset:0;background-image:linear-gradient(rgba(0,0,0,.03) 1px,transparent 1px),linear-gradient(90deg,rgba(0,0,0,.03) 1px,transparent 1px);background-size:40px 40px}
.login-left>*{position:relative;z-index:1}
.brand{display:flex;align-items:center;gap:10px;margin-bottom:40px}
.brand svg{width:36px;height:36px}.brand span{font-size:1.4rem;font-weight:700;letter-spacing:-.5px}
.login-left h2{font-size:2.2rem;font-weight:700;line-height:1.2;letter-spacing:-1px;margin-bottom:16px}
.hl{color:var(--o)}
.login-left .desc{color:var(--d);font-size:.95rem;line-height:1.6;margin-bottom:40px}
.tc{background:var(--bg);border:1px solid var(--b);border-radius:12px;overflow:hidden;box-shadow:0 4px 24px rgba(0,0,0,.06);margin-bottom:40px}
.td{display:flex;gap:6px;padding:12px 16px;border-bottom:1px solid var(--b)}
.td span{width:10px;height:10px;border-radius:50%}
.td span:nth-child(1){background:#ff5f57}.td span:nth-child(2){background:#febc2e}.td span:nth-child(3){background:#28c840}
.tcd{padding:16px 20px;font-family:'JetBrains Mono',monospace;font-size:.8rem;line-height:1.8;color:var(--d)}
.tcd .p{color:var(--t);font-weight:500}.tcd .c{color:var(--o)}
.mets{display:flex;gap:32px}
.met .v{font-size:1.5rem;font-weight:700}.met .v .u{color:var(--o);font-weight:600}
.met .l{font-size:.75rem;color:var(--m);margin-top:2px}
.login-right{width:480px;display:flex;flex-direction:column;justify-content:center;padding:48px;position:relative}
.login-right::before{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at top right,rgba(255,92,0,.04),transparent 60%)}
.login-right>*{position:relative;z-index:1}
.bsm{display:flex;align-items:center;gap:8px;justify-content:center;margin-bottom:32px}
.bsm svg{width:28px;height:28px}.bsm span{font-size:1.1rem;font-weight:700}
.login-right h1{font-size:1.75rem;font-weight:700;margin-bottom:8px}
.login-right .sub{color:var(--d);font-size:.9rem;margin-bottom:32px}
.fg{margin-bottom:16px}
.fg label{display:block;font-size:.8rem;font-weight:500;color:var(--d);margin-bottom:6px}
.iw{position:relative}
.iw input{width:100%;padding:12px 16px 12px 44px;border:1px solid var(--b);border-radius:12px;font-size:.9rem;font-family:inherit;color:var(--t);outline:none;transition:border-color .2s,box-shadow .2s}
.iw input:focus{border-color:var(--o);box-shadow:0 0 0 3px rgba(255,92,0,.1)}
.iw input::placeholder{color:var(--m)}
.iw .ic{position:absolute;left:14px;top:50%;transform:translateY(-50%);color:var(--m)}
.bl{width:100%;padding:14px;background:var(--o);color:#fff;border:none;border-radius:12px;font-size:.95rem;font-weight:600;font-family:inherit;cursor:pointer;transition:background .2s;margin-top:8px}
.bl:hover{background:var(--oh)}.bl:disabled{opacity:.5;cursor:not-allowed}
.em{color:var(--r);font-size:.8rem;margin-top:12px;display:none}
.lf{margin-top:24px;text-align:center;font-size:.8rem;color:var(--m)}.lf a{color:var(--o);text-decoration:none;font-weight:500}
/* Dashboard Layout */
.dashboard{display:none;min-height:100vh}
.dl{display:flex;min-height:100vh}
.sb{width:220px;background:var(--bg);border-right:1px solid var(--b);display:flex;flex-direction:column;flex-shrink:0;position:fixed;left:0;top:0;bottom:0;z-index:10}
.sbh{padding:16px 20px;display:flex;align-items:center;gap:10px;border-bottom:1px solid var(--b)}
.sbh svg{width:28px;height:28px}.sbh span{font-size:1rem;font-weight:700}
.sbu{padding:12px 20px;font-size:.8rem;color:var(--d);border-bottom:1px solid var(--b)}
.sbn{flex:1;padding:8px}
.si{display:flex;align-items:center;gap:10px;padding:10px 12px;border-radius:8px;font-size:.85rem;font-weight:500;color:var(--d);cursor:pointer;transition:all .15s;text-decoration:none}
.si:hover{background:var(--bg2);color:var(--t)}.si.active{background:var(--ol);color:var(--o)}
.si svg{width:18px;height:18px;flex-shrink:0}
.sbb{padding:8px;border-top:1px solid var(--b)}
.sbb .si{font-size:.8rem;padding:8px 12px}
.mn{flex:1;margin-left:220px;display:flex;flex-direction:column;min-height:100vh}
.mh{padding:20px 32px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--b);background:var(--bg)}
.mh h1{font-size:1.3rem;font-weight:700;display:flex;align-items:center;gap:10px}
.mc{padding:24px 32px;flex:1}
/* List View */
.tb{display:flex;gap:12px;margin-bottom:16px;align-items:center}
.sx{flex:1;position:relative}
.sx input{width:100%;padding:10px 16px 10px 40px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);background:var(--bg);outline:none}
.sx input:focus{border-color:var(--o)}
.sx input::placeholder{color:var(--m)}
.sx svg{position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--m);width:16px;height:16px}
.fb{padding:10px 16px;border:1px solid var(--b);border-radius:10px;background:var(--bg);font-size:.85rem;font-family:inherit;color:var(--t);cursor:pointer;display:flex;align-items:center;gap:6px}
.sr{display:flex;gap:16px;margin-bottom:20px;font-size:.85rem;font-weight:500}
.sr .sl{color:var(--d)}.sr .sv{font-weight:700}.sr .sv.gn{color:var(--gt)}
/* Table */
.dt{width:100%;border-collapse:collapse;font-size:.85rem;background:var(--bg);border:1px solid var(--b);border-radius:10px;overflow:hidden}
.dt th{padding:12px 16px;text-align:left;font-weight:600;font-size:.75rem;text-transform:uppercase;color:var(--d);background:var(--bg2);border-bottom:1px solid var(--b)}
.dt td{padding:12px 16px;border-bottom:1px solid var(--b);vertical-align:middle}
.dt tr:last-child td{border-bottom:none}
.dt tr:hover td{background:var(--bg2)}
.dt .nl{color:var(--o);font-weight:500;cursor:pointer;text-decoration:none}
.dt .nl:hover{text-decoration:underline}
/* Badges */
.badge{display:inline-block;padding:3px 10px;border-radius:20px;font-size:.75rem;font-weight:600}
.badge.running{background:var(--gb);color:var(--gt)}.badge.stopped{background:var(--rb);color:var(--rt)}
.badge.plan{background:var(--pb);color:var(--pt)}.badge.pro{background:var(--ol);color:var(--o)}
.vt{font-family:'JetBrains Mono',monospace;font-size:.75rem;color:var(--d)}
/* Buttons */
.btn-o{background:var(--o);color:#fff;border:none;border-radius:8px;padding:8px 16px;font-size:.85rem;font-weight:600;font-family:inherit;cursor:pointer;display:inline-flex;align-items:center;gap:6px;transition:background .15s}
.btn-o:hover{background:var(--oh)}
.btn-g{background:var(--bg);color:var(--t);border:1px solid var(--b);border-radius:8px;padding:8px 16px;font-size:.85rem;font-weight:500;font-family:inherit;cursor:pointer;display:inline-flex;align-items:center;gap:6px;transition:all .15s}
.btn-g:hover{background:var(--bg2);border-color:var(--m)}
.btn-r{color:var(--r);background:none;border:none;font-size:.8rem;font-weight:500;cursor:pointer;font-family:inherit;display:inline-flex;align-items:center;gap:4px}
.btn-r:hover{text-decoration:underline}
/* Detail Header */
.bc{font-size:.8rem;color:var(--d);margin-bottom:8px}
.bc a{color:var(--d);text-decoration:none;cursor:pointer}.bc a:hover{color:var(--o)}
.dh{display:flex;align-items:flex-start;justify-content:space-between;margin-bottom:4px}
.dh h2{font-size:1.5rem;font-weight:700;line-height:1.3}
.dh-meta{font-family:'JetBrains Mono',monospace;font-size:.85rem;color:var(--d);margin-bottom:20px;display:flex;align-items:center;gap:8px}
.dh-actions{display:flex;gap:8px;align-items:center;flex-shrink:0}
/* Tabs */
.tabs{display:flex;gap:0;border-bottom:2px solid var(--b);margin-bottom:24px}
.tab{padding:10px 20px;font-size:.85rem;font-weight:500;color:var(--d);cursor:pointer;border-bottom:2px solid transparent;margin-bottom:-2px;transition:all .15s;white-space:nowrap}
.tab:hover{color:var(--t)}.tab.active{color:var(--o);border-bottom-color:var(--o)}
/* Stat Cards */
.cards{display:grid;grid-template-columns:repeat(4,1fr);gap:16px;margin-bottom:24px}
.card{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:20px}
.card .card-label{font-size:.8rem;color:var(--d);display:flex;align-items:center;gap:6px;margin-bottom:8px}
.card .card-val{font-size:1.3rem;font-weight:700}
.card .card-sub{font-size:.75rem;color:var(--d);margin-top:4px}
.card .bar{height:4px;background:var(--bg3);border-radius:4px;margin-top:10px;overflow:hidden}
.card .bar-fill{height:100%;background:var(--g);border-radius:4px}
/* Section Box */
.sbox{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:20px;margin-bottom:20px}
.sbox h3{font-size:1rem;font-weight:700;margin-bottom:4px}
.sbox .sbox-desc{font-size:.85rem;color:var(--d);margin-bottom:16px}
/* Detail Grid */
.detail-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px 40px}
.detail-item{padding:8px 0;border-bottom:1px solid var(--bg3)}
.detail-item .di-label{font-size:.75rem;color:var(--d);text-transform:uppercase;letter-spacing:.3px;font-weight:600}
.detail-item .di-value{font-size:.9rem;font-weight:500;margin-top:2px}
/* Config Form */
.config-section{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:24px;margin-bottom:20px}
.config-section h3{font-size:.8rem;font-weight:600;text-transform:uppercase;letter-spacing:.5px;color:var(--d);margin-bottom:16px}
.config-row{display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:16px}
.config-row .fg{margin-bottom:0}
.fg select,.fg input[type=text],.fg input[type=password]{width:100%;padding:10px 14px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);outline:none;background:var(--bg)}
.fg select:focus,.fg input:focus{border-color:var(--o)}
/* Empty State */
.empty{text-align:center;padding:60px 40px;background:var(--bg);border:1px solid var(--b);border-radius:12px}
.empty .empty-icon{font-size:2.5rem;margin-bottom:12px;color:var(--m)}
.empty h4{font-size:1rem;margin-bottom:8px}
.empty p{font-size:.85rem;color:var(--d);margin-bottom:20px}
/* Role Dropdown */
.role-sel{padding:6px 10px;border:1px solid var(--b);border-radius:8px;font-size:.8rem;font-family:inherit;color:var(--t);cursor:pointer;background:var(--bg);min-width:110px;outline:none}
.role-sel:focus{border-color:var(--o)}
/* Modal */
.modal-bg{display:none;position:fixed;inset:0;background:rgba(0,0,0,.4);z-index:100;align-items:center;justify-content:center}
.modal-bg.show{display:flex}
.modal{background:var(--bg);border-radius:12px;padding:24px;width:440px;max-width:90vw;box-shadow:0 20px 60px rgba(0,0,0,.15)}
.modal h3{font-size:1.1rem;font-weight:700;margin-bottom:16px}
.modal .fg input,.modal .fg select{width:100%;padding:10px 14px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);outline:none}
.modal .fg input:focus,.modal .fg select:focus{border-color:var(--o)}
.modal .actions{display:flex;gap:8px;justify-content:flex-end;margin-top:20px}
.modal .btn-cancel{background:var(--bg2);border:1px solid var(--b);border-radius:8px;padding:8px 16px;font-size:.85rem;cursor:pointer;font-family:inherit}
/* Warning */
.warn{display:flex;align-items:center;gap:8px;padding:12px 16px;background:var(--ol);border:1px solid #fed7aa;border-radius:10px;font-size:.85rem;color:#9a3412;margin-bottom:16px}
@media(max-width:900px){.login-screen{flex-direction:column}.login-left{display:none}.login-right{width:100%;min-height:100vh}.sb{display:none}.mn{margin-left:0}}
@media(max-width:768px){.tabs{overflow-x:auto;-webkit-overflow-scrolling:touch;flex-wrap:nowrap;gap:0;padding-bottom:4px}.tab{white-space:nowrap;flex-shrink:0;padding:8px 12px;font-size:.8rem}.config-section{padding:12px}.config-row{flex-direction:column;gap:8px}.dh h2{font-size:1.1rem}.dh-meta{font-size:.75rem}.mn{padding:12px}#headerActions{flex-wrap:wrap;gap:6px}#headerActions a,#headerActions button{font-size:.75rem;padding:6px 10px}.dt{font-size:.8rem}.dt th,.dt td{padding:6px 8px}}
</style>
</head>
<body>
<!-- LOGIN -->
<div class="login-screen" id="loginView">
  <div class="login-left">
    <div class="brand"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
    <h2>Deploy &amp; manage<br>AI agents with <span class="hl">the official<br>OpenFang runtime</span></h2>
    <p class="desc">Self-service portal for team members. Manage your tenants, view analytics, and collaborate securely.</p>
    <div class="tc"><div class="td"><span></span><span></span><span></span></div><div class="tcd"><div><span class="p">$</span> <span class="c">openfang serve</span></div><div>booted in &lt;200ms</div><div>hands 7 active</div><div>gateway ready :3000</div></div></div>
    <div class="mets"><div class="met"><div class="v">32 <span class="u">MB</span></div><div class="l">Binary</div></div><div class="met"><div class="v">180<span class="u">ms</span></div><div class="l">Cold Start</div></div><div class="met"><div class="v">26<span class="u">+</span></div><div class="l">Providers</div></div></div>
  </div>
  <div class="login-right">
    <div class="bsm"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
    <h1>Welcome back</h1>
    <p class="sub">Sign in to manage your tenants and agents.</p>
    <div class="fg"><label>Email address</label><div class="iw"><svg class="ic" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg><input type="email" id="loginEmail" placeholder="email" autocomplete="email@domain.com"></div></div>
    <div class="fg"><label>Password</label><div class="iw"><svg class="ic" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg><input type="password" id="loginPass" placeholder="Enter your password" autocomplete="current-password" onkeydown="if(event.key==='Enter')doLogin()"></div></div>
    <button class="bl" id="loginBtn" onclick="doLogin()">Sign In</button>
    <div class="em" id="loginError"></div>
    <div class="lf">System admins can use their <a href="javascript:void(0)">API key</a> as password</div>
  </div>
</div>

<!-- DASHBOARD -->
<div class="dashboard" id="dashView">
  <div class="dl">
    <div class="sb">
      <div class="sbh"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
      <div class="sbu" id="sbUser">Admin</div>
      <div class="sbn">
        <a class="si active" onclick="showPage('tenants')"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg>Tenants</a>
        <a class="si" onclick="showPage('members')" id="membersNav" style="display:none"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4-4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>Members</a>
        <a class="si" onclick="showPage('users')" id="usersNav" style="display:none"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>Users</a>
        <a class="si" onclick="showPage('plans')" id="plansNav" style="display:none"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>Plans</a>
      </div>
      <div class="sbb"><a class="si" onclick="doLogout()"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9"/></svg>Logout</a></div>
    </div>
    <div class="mn">
      <div class="mh"><h1 id="pageTitle">Tenants</h1><div id="headerActions"></div></div>
      <div class="mc" id="mainContent"></div>
    </div>
  </div>
</div>

<div class="modal-bg" id="addMemberModal">
  <div class="modal">
    <h3>Add Member</h3>
    <div class="fg"><label>Email</label><input type="email" id="amEmail" placeholder="user@example.com"></div>
    <div class="fg"><label>Display Name</label><input type="text" id="amName" placeholder="John Doe"></div>
    <div class="fg"><label>Role</label><select id="amRole"><option value="viewer">Viewer</option><option value="contributor">Contributor</option><option value="manager">Manager</option><option value="owner">Owner</option></select></div>
    <div class="fg"><label>Password (optional)</label><input type="password" id="amPass" placeholder="Min 4 chars for portal login"></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('addMemberModal')">Cancel</button><button class="btn-o" onclick="doAddMember()">Add Member</button></div>
  </div>
</div>

<!-- Add Channel Modal -->
<div class="modal-bg" id="addChannelModal">
  <div class="modal">
    <h3>Add Channel</h3>
    <div class="fg"><label>Channel Type</label><select id="acType"><option value="">Select type...</option><option value="telegram">Telegram</option><option value="discord">Discord</option><option value="slack">Slack</option><option value="whatsapp">WhatsApp</option><option value="signal">Signal</option><option value="matrix">Matrix</option><option value="email">Email</option><option value="zalo">Zalo</option><option value="web">Web Widget</option></select></div>
    <div class="fg"><label>Display Name (optional)</label><input type="text" id="acName" placeholder="e.g. My Telegram Bot"></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('addChannelModal')">Cancel</button><button class="btn-o" onclick="addChannel()">Add Channel</button></div>
  </div>
</div>

<!-- Create User Modal -->
<div class="modal-bg" id="createUserModal">
  <div class="modal">
    <h3>Create User</h3>
    <div class="fg"><label>Email</label><input type="email" id="cuEmail" placeholder="user@example.com"></div>
    <div class="fg"><label>Display Name</label><input type="text" id="cuName" placeholder="John Doe"></div>
    <div class="fg"><label>Password</label><input type="password" id="cuPass" placeholder="Min 4 characters"></div>
    <div class="fg"><label>Role</label><select id="cuRole"><option value="user">User</option><option value="admin">Admin</option></select></div>
    <div class="fg"><label>Plan</label><select id="cuPlan"></select></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createUserModal')">Cancel</button><button class="btn-o" onclick="doCreateUser()">Create User</button></div>
  </div>
</div>

<!-- Create Plan Modal -->
<div class="modal-bg" id="createPlanModal">
  <div class="modal">
    <h3>Create Plan</h3>
    <div class="fg"><label>Plan Name</label><input type="text" id="cpName" placeholder="e.g. Starter"></div>
    <div class="config-row"><div class="fg"><label>Messages/Day</label><input type="number" id="cpMsg" value="500"></div><div class="fg"><label>Max Channels</label><input type="number" id="cpCh" value="5"></div></div>
    <div class="config-row"><div class="fg"><label>Max Members</label><input type="number" id="cpMem" value="10"></div><div class="fg"><label>Max Tenants</label><input type="number" id="cpTen" value="5"></div></div>
    <div class="fg"><label>Price Label</label><input type="text" id="cpPrice" placeholder="e.g. $19/mo"></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createPlanModal')">Cancel</button><button class="btn-o" onclick="doCreatePlan()">Create Plan</button></div>
  </div>
</div>

<!-- Create Tenant Modal -->
<div class="modal-bg" id="createTenantModal">
  <div class="modal">
    <h3>Create Tenant</h3>
    <div class="fg"><label>Tenant Name</label><input type="text" id="ctName" placeholder="e.g. My AI Bot"></div>
    <div class="config-row"><div class="fg"><label>Provider</label><select id="ctProvider"><option value="groq">Groq</option><option value="openai">OpenAI</option><option value="anthropic">Anthropic</option><option value="openrouter">OpenRouter</option><option value="deepseek">DeepSeek</option><option value="ollama">Ollama</option><option value="gemini">Gemini</option></select></div><div class="fg"><label>Model</label><input type="text" id="ctModel" value="llama-3.3-70b-versatile"></div></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createTenantModal')">Cancel</button><button class="btn-o" onclick="doCreateMyTenant()">Create Tenant</button></div>
  </div>
</div>

<script>
let S=null,T=[],D=null,CTab='overview';
const ROLES=['Owner','Manager','Contributor','Viewer'];
const INF=4294967295;
function api(m,p,b){const o={method:m,headers:{}};if(S)o.headers.Authorization='Bearer '+S.token;if(b){o.headers['Content-Type']='application/json';o.body=JSON.stringify(b)}return fetch(p,o).then(r=>r.json()).catch(e=>({error:e.message}))}
function fmt(v){return v>=INF?'Unlimited':v}
function fmtDate(d){if(!d)return'-';try{return new Date(d).toLocaleDateString('vi-VN',{day:'2-digit',month:'2-digit',year:'numeric',hour:'2-digit',minute:'2-digit'})}catch(e){return d}}

// Auth
async function doLogin(){const e=document.getElementById('loginEmail').value.trim(),p=document.getElementById('loginPass').value,err=document.getElementById('loginError');err.style.display='none';if(!e||!p){err.textContent='Please fill in all fields';err.style.display='block';return}document.getElementById('loginBtn').disabled=true;try{const d=await api('POST','/api/portal/login',{email:e,password:p});if(d.error){err.textContent=d.error;err.style.display='block';return}S=d;localStorage.setItem('ps',JSON.stringify(d));showDash()}catch(x){err.textContent='Connection error';err.style.display='block'}finally{document.getElementById('loginBtn').disabled=false}}
function doLogout(){S=null;localStorage.removeItem('ps');document.getElementById('loginView').style.display='flex';document.getElementById('dashView').style.display='none'}
async function showDash(){document.getElementById('loginView').style.display='none';document.getElementById('dashView').style.display='block';document.getElementById('sbUser').textContent=S.display_name||S.email;if(S.role==='admin'){document.getElementById('membersNav').style.display='';document.getElementById('usersNav').style.display='';document.getElementById('plansNav').style.display=''}await loadT();showPage('tenants')}
async function loadT(){const d=await api('GET','/api/portal/tenants');T=d.tenants||[]}

// Navigation
function showPage(p){D=null;document.querySelectorAll('.sbn .si').forEach(el=>el.classList.remove('active'));document.getElementById('headerActions').innerHTML='';history.pushState({page:p},'','/');
if(p==='tenants'){document.querySelector('.sbn .si:first-child').classList.add('active');document.getElementById('pageTitle').innerHTML='<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:24px;height:24px"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg> Tenants';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openCreateTenantModal()">+ Create Tenant</button>';renderList()}
else if(p==='members'){document.getElementById('membersNav').classList.add('active');document.getElementById('pageTitle').textContent='Members';renderMembers()}
else if(p==='users'){document.getElementById('usersNav').classList.add('active');document.getElementById('pageTitle').textContent='Users';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openCreateUserModal()">+ Create User</button>';renderUsers()}
else if(p==='plans'){document.getElementById('plansNav').classList.add('active');document.getElementById('pageTitle').textContent='Service Plans';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openModal(\"createPlanModal\")">+ Create Plan</button>';renderPlans()}}

// Tenant List
function renderList(){
  const run=T.filter(t=>t.status==='running').length;
  const rows=T.map(t=>`<tr><td><a class="nl" onclick="openDetail('${t.id}')">${t.name}</a></td><td class="vt">${t.slug}</td><td><span class="badge ${t.status}">${t.status==='running'?'Running':'Stopped'}</span></td><td><span class="badge pro">${(t.plan||'free').charAt(0).toUpperCase()+(t.plan||'free').slice(1)}</span></td><td class="vt">${t.version||'-'}</td><td style="color:var(--d)">${fmtDate(t.created_at)}</td><td>${S.role==='admin'?`<button class="btn-r" onclick="event.stopPropagation();doDeleteFromList('${t.id}','${t.name.replace(/'/g,"\\'")}')">Delete</button>`:''}</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="tb"><div class="sx"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg><input type="text" id="si2" placeholder="Search by name or slug..." oninput="filterT()"></div><button class="fb" onclick="toggleF()"><span id="fl">All statuses</span> &#9662;</button></div><div class="sr"><span class="sl">Total: <span class="sv">${T.length}</span></span><span class="sl">Running: <span class="sv gn">${run}</span></span></div><table class="dt" id="tt"><thead><tr><th>Name</th><th>Slug</th><th>Status</th><th>Plan</th><th>Version</th><th>Created</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
let sf='all';
function toggleF(){sf=sf==='all'?'running':sf==='running'?'stopped':'all';document.getElementById('fl').textContent=sf==='all'?'All statuses':sf==='running'?'Running only':'Stopped only';filterT()}
function filterT(){const q=(document.getElementById('si2')?.value||'').toLowerCase();document.querySelectorAll('#tt tbody tr').forEach((r,i)=>{const t=T[i];if(!t)return;const ms=!q||t.name.toLowerCase().includes(q)||t.slug.toLowerCase().includes(q);const mf=sf==='all'||(sf==='running'&&t.status==='running')||(sf==='stopped'&&t.status!=='running');r.style.display=ms&&mf?'':'none'})}

// Tenant Detail
async function openDetail(id){
  const d=await api('GET','/api/portal/tenants/'+id);if(d.error)return;D=d;CTab='overview';history.pushState({page:'detail',id:id},d.name,'/'+id);renderDetailPage();
}
function renderDetailPage(){
  if(!D)return;const t=D;const isAdmin=S.role==='admin';
  const isOwner=!isAdmin&&(t.members||[]).some(m=>m.email.toLowerCase()===S.email.toLowerCase()&&(m.role==='owner'||m.role==='manager'));
  const canEdit=isAdmin||isOwner;
  document.getElementById('pageTitle').innerHTML=`<span>${t.name}</span>`;
  const ha=canEdit?`<a class="btn-o" href="/access/?t=${t.access_token||''}" target="_blank">Open Dashboard</a><button class="btn-g" onclick="cloneTenant()">Clone</button><button class="btn-g" onclick="doRestart()">Restart</button><button class="btn-g" onclick="doStop()">Stop</button><button class="btn-r" style="padding:8px 16px;border:1px solid var(--b);border-radius:8px" onclick="doDeleteTenant()">Delete</button>`:'';
  document.getElementById('headerActions').innerHTML=ha;
  renderDetailBody();
}
async function renderDetailBody(){
  if(!D)return;const t=D;const isAdmin=S.role==='admin';
  const isOwner=!isAdmin&&(t.members||[]).some(m=>m.email.toLowerCase()===S.email.toLowerCase()&&(m.role==='owner'||m.role==='manager'));
  const canEdit=isAdmin||isOwner;
  const bc=`<div class="bc"><a onclick="showPage('tenants')">Tenants</a> &gt; ${t.name}</div>`;
  const header=`<div class="dh"><h2>${t.name}</h2></div><div class="dh-meta"><span>${t.slug}</span> &middot; <span class="badge ${t.status}">${t.status==='running'?'Running':'Stopped'}</span></div>`;
  const TABS=['Overview','Config','Channels','Agent','Assistant','Usage','Members','History'];
  const tabsHtml=`<div class="tabs">${TABS.map(tb=>`<div class="tab${CTab===tb.toLowerCase()?' active':''}" onclick="CTab='${tb.toLowerCase()}';renderDetailBody()">${tb}</div>`).join('')}</div>`;
  let body='';
  if(CTab==='overview') body=renderOverview(t);
  else if(CTab==='config') body=await renderConfig(t);
  else if(CTab==='channels') body=await renderChannels(t);
  else if(CTab==='agent') body=await renderAgent(t,canEdit);
  else if(CTab==='assistant') body=renderAssistant(t);
  else if(CTab==='usage') body=renderUsage(t);
  else if(CTab==='members') body=renderMembersTab(t,isAdmin);
  else if(CTab==='history') body=await renderHistory(t);
  document.getElementById('mainContent').innerHTML=bc+header+tabsHtml+body;
  if(CTab==='config') loadModelsForProvider();
}

// Tab: Overview
function renderOverview(t){
  const chCount=(t.channels||[]).length;
  let html=`<div class="cards">
    <div class="card"><div class="card-label">Status</div><div class="card-val"><span class="badge ${t.status}" style="font-size:.85rem;padding:4px 14px">${t.status==='running'?'Running':'Stopped'}</span></div></div>
    <div class="card"><div class="card-label">Provider</div><div class="card-val" style="font-size:1rem;text-transform:capitalize">${t.provider||'-'}</div><div class="card-sub">${t.model||''}</div></div>
    <div class="card"><div class="card-label">Channels</div><div class="card-val">${chCount} / ${fmt(t.max_channels)}</div></div>
    <div class="card"><div class="card-label">Messages</div><div class="card-val">${t.messages_today} today</div><div class="card-sub">Limit: ${fmt(t.max_messages_per_day)}/day</div></div>
  </div>`;
  // Magic Access Link
  html+=`<div class="sbox"><h3>Magic Access Link</h3><div class="sbox-desc">One-time link for instant dashboard access. Share it directly or send via email.</div>`;
  if(t.access_token){html+=`<div style="display:flex;align-items:center;gap:8px"><code style="flex:1;padding:10px 14px;background:var(--bg2);border:1px solid var(--b);border-radius:8px;font-size:.8rem;word-break:break-all">${location.origin}/access/?t=${t.access_token}</code><button class="btn-g" onclick="navigator.clipboard.writeText('${location.origin}/access/?t=${t.access_token}')">Copy</button></div>`}
  else{html+=`<button class="btn-g">Generate Access Link</button>`}
  html+=`</div>`;
  // Tenant Details
  html+=`<div class="sbox"><h3>Tenant Details</h3><div class="detail-grid">
    <div class="detail-item"><div class="di-label">ID</div><div class="di-value vt" style="font-size:.85rem">${t.id}</div></div>
    <div class="detail-item"><div class="di-label">Subdomain</div><div class="di-value"><span class="vt" style="color:var(--o)">${t.slug}.${location.hostname}</span></div></div>
    <div class="detail-item"><div class="di-label">Plan</div><div class="di-value"><span class="badge pro">${(t.plan||'free').charAt(0).toUpperCase()+(t.plan||'free').slice(1)}</span> - ${fmt(t.max_messages_per_day)} msg/day, ${fmt(t.max_channels)} ch, ${fmt(t.max_members)} members</div></div>
    <div class="detail-item"><div class="di-label">Temperature</div><div class="di-value">${t.temperature}</div></div>
    <div class="detail-item"><div class="di-label">Version</div><div class="di-value vt">${t.version||'-'}</div></div>
    <div class="detail-item"><div class="di-label">Created</div><div class="di-value">${fmtDate(t.created_at)}</div></div>
  </div></div>`;
  return html;
}

// Tab: Config
async function renderConfig(t){
  const isAdmin=S.role==='admin';
  const isOwner=(t.members||[]).some(m=>m.email.toLowerCase()===S.email.toLowerCase()&&(m.role==='owner'||m.role==='manager'));
  const canEdit=isAdmin||isOwner;
  const dis=canEdit?'':'disabled';
  // Load providers from system API
  let provOpts=`<option value="${t.provider}">${t.provider}</option>`;
  try{const pd=await api('GET','/api/portal/system/providers');const provs=pd.providers||[];
    provOpts=provs.map(p=>`<option value="${p.id}"${t.provider===p.id?' selected':''}>${p.display_name}${p.auth_status==='configured'?' [OK]':p.auth_status==='not_required'?' [Local]':' [No Key]'}</option>`).join('');
  }catch(e){}
  let html=`<div class="config-section"><h3>AI Provider</h3>
    <div class="config-row"><div class="fg"><label>Provider</label><select id="cfgProvider" ${dis} onchange="loadModelsForProvider()">${provOpts}</select></div><div class="fg"><label>Model</label><select id="cfgModel" ${dis}><option value="${t.model||''}">${t.model||'Select model'}</option></select></div></div>
    <div class="config-row"><div class="fg"><label>Temperature</label><input type="text" id="cfgTemp" value="${t.temperature}" ${dis} style="width:120px"></div><div class="fg"><label>API Key</label><input type="password" id="cfgApiKey" value="${t.api_key||''}" placeholder="Provider API key" ${dis}></div></div>`;
  if(canEdit){html+=`<div style="margin-top:12px"><button class="btn-o" onclick="saveConfig()">Save Config</button></div>`}
  else{html+=`<div class="warn">Configuration changes are managed by the tenant owner or admin.</div>`}
  html+=`</div>
  <div class="config-section"><h3>Quotas</h3>
    <div class="config-row"><div class="fg"><label>Messages per Day</label><input type="text" value="${fmt(t.max_messages_per_day)}" disabled></div><div class="fg"><label>Max Channels</label><input type="text" value="${fmt(t.max_channels)}" disabled></div></div>
    <div class="fg"><label>Max Members</label><input type="text" value="${fmt(t.max_members)}" disabled style="width:200px"></div>
  </div>`;
  return html;
}
async function loadModelsForProvider(){
  const prov=document.getElementById('cfgProvider').value;
  const sel=document.getElementById('cfgModel');
  sel.innerHTML='<option>Loading...</option>';
  try{const d=await api('GET','/api/portal/system/models?provider='+prov);const ms=d.models||[];
    sel.innerHTML=ms.map(m=>`<option value="${m.id}"${D&&D.model===m.id?' selected':''}>${m.display_name} (${m.id})</option>`).join('');
    if(ms.length===0)sel.innerHTML='<option value="">No models found</option>';
  }catch(e){sel.innerHTML='<option value="">Error loading models</option>'}
}

// Tab: Channels
let configuringChannel=null;
async function renderChannels(t){
  const isAdmin=S.role==='admin';
  const isOwner=(t.members||[]).some(m=>m.email.toLowerCase()===S.email.toLowerCase()&&(m.role==='owner'||m.role==='manager'));
  const canEdit=isAdmin||isOwner;
  const tenantCh=t.channels||[];
  // Load system channels
  let sysCh=[];
  try{const d=await api('GET','/api/portal/system/channels');sysCh=d.channels||[]}catch(e){}
  // Map tenant channels with system info
  const addBtn=canEdit?`<button class="btn-o" onclick="openAddSystemChannel()">+ Add Channel</button>`:'';
  const cnt=`<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px"><div><h3 style="font-size:1rem;font-weight:700">Channels</h3><p style="font-size:.8rem;color:var(--d)">${tenantCh.length} / ${fmt(t.max_channels)} channels connected</p></div>${addBtn}</div>`;
  // Active channels
  let html=cnt;
  if(tenantCh.length>0){
    const rows=tenantCh.map(c=>{const sys=sysCh.find(s=>s.name===c.channel_type)||{};
      const cfgKeys=Object.keys(c.config||{}).filter(k=>c.config[k]);
      const hasConfig=cfgKeys.length>0;
      const statusBadge=hasConfig?'<span class="badge running">Configured</span>':'<span class="badge stopped">Not Configured</span>';
      const cfgBtn=canEdit?`<button class="btn-g" onclick="configureChannel('${c.name}','${c.channel_type}')">Configure</button>`:'';
      const del=canEdit?`<button class="btn-r" onclick="removeChannel('${c.name}')">Remove</button>`:'';
      let row=`<tr><td style="text-transform:capitalize;font-weight:500">${sys.display_name||c.channel_type||'-'}</td><td>${c.name||c.channel_type||'-'}</td><td>${statusBadge}</td><td>${cfgBtn} ${del}</td></tr>`;
      // If configuring this channel, show config form below
      if(configuringChannel===c.name){
        const fields=(sys.fields||[]).filter(f=>!f.advanced);
        let formHtml=`<tr><td colspan="4" style="background:var(--bg2);padding:16px"><div style="margin-bottom:8px;font-weight:600">Configure ${sys.display_name||c.channel_type}</div><div style="display:flex;flex-direction:column;gap:8px">`;
        fields.forEach(f=>{
          const val=(c.config||{})[f.key]||'';
          const inputType=f.type==='secret'?'password':'text';
          formHtml+=`<div style="display:flex;align-items:center;gap:8px"><label style="min-width:140px;font-size:.85rem;font-weight:500">${f.label}${f.required?' *':''}</label><input type="${inputType}" class="chcfg" data-key="${f.key}" value="${val}" placeholder="${f.placeholder||''}" style="flex:1;max-width:400px"></div>`;
        });
        formHtml+=`</div><div style="margin-top:12px;display:flex;gap:8px"><button class="btn-o" onclick="saveChannelConfig('${c.name}')">Save Config</button><button class="btn-cancel" onclick="configuringChannel=null;renderDetailBody()">Cancel</button></div></td></tr>`;
        row+=formHtml;
      }
      return row;
    }).join('');
    html+=`<table class="dt"><thead><tr><th>Type</th><th>Name</th><th>Status</th>${canEdit?'<th>Actions</th>':''}</tr></thead><tbody>${rows}</tbody></table>`;
  } else {
    html+=`<div class="empty"><div class="empty-icon">(( ))</div><h4>No channels connected</h4><p>Connect a messaging platform to start receiving messages.</p></div>`;
  }
  // Available system channels
  if(canEdit && sysCh.length>0){
    html+=`<div style="margin-top:24px"><h3 style="font-size:1rem;font-weight:700;margin-bottom:12px">Available Channels (${sysCh.length})</h3>`;
    const cats=[...new Set(sysCh.map(c=>c.category))];
    cats.forEach(cat=>{
      const chs=sysCh.filter(c=>c.category===cat);
      html+=`<div style="margin-bottom:12px"><div style="font-size:.8rem;font-weight:600;color:var(--d);text-transform:uppercase;margin-bottom:6px">${cat} (${chs.length})</div>`;
      html+=`<div style="display:flex;flex-wrap:wrap;gap:8px">`;
      chs.forEach(c=>{const connected=tenantCh.some(tc=>tc.channel_type===c.name);
        const badge=connected?'running':c.configured?'plan':'stopped';
        const label=connected?'Connected':c.configured?'Available':'Not Configured';
        html+=`<div style="padding:6px 12px;background:var(--bg2);border:1px solid var(--b);border-radius:8px;font-size:.8rem;display:flex;align-items:center;gap:6px"><span style="font-weight:500">${c.display_name}</span><span class="badge ${badge}" style="font-size:.65rem;padding:2px 6px">${label}</span>`;if(!connected&&canEdit)html+=`<button class="btn-g" style="padding:2px 8px;font-size:.7rem" onclick="addSystemChannel('${c.name}','${c.display_name}')">Add</button>`;html+=`</div>`;
      });html+=`</div></div>`;
    });html+=`</div>`;
  }
  return html;
}
function openAddSystemChannel(){CTab='channels';renderDetailBody()}
async function addSystemChannel(type,name){const body={channel_type:type,name:name};const d=await api('POST','/api/portal/tenants/'+D.id+'/channels',body);if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);configuringChannel=name;renderDetailBody()}else{alert(d.error||'Failed')}}
function configureChannel(name,type){configuringChannel=name;renderDetailBody()}
async function saveChannelConfig(channelName){
  const inputs=document.querySelectorAll('.chcfg');
  const config={};
  inputs.forEach(i=>{if(i.value)config[i.dataset.key]=i.value});
  const d=await api('PUT','/api/portal/tenants/'+D.id+'/channels/config',{channel_name:channelName,config});
  if(d.ok){configuringChannel=null;D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody();alert('Channel config saved!')}else{alert(d.error||'Failed')}
}

// Tab: Agent (System Prompt, Skills, Hands, Language, Webhook)
const PROMPT_TEMPLATES=[
  {name:'Custom',prompt:''},
  {name:'Customer Support',prompt:'You are a friendly and professional customer support agent. Always greet the customer, listen carefully to their issue, provide clear solutions, and ensure satisfaction before ending the conversation. Use a warm and helpful tone.'},
  {name:'Sales Agent',prompt:'You are an enthusiastic sales agent. Your goal is to understand customer needs, recommend the best products or services, handle objections professionally, and guide customers toward making a purchase. Be persuasive but never pushy.'},
  {name:'Knowledge Base',prompt:'You are a knowledgeable assistant that answers questions based on the company\'s knowledge base. Provide accurate, detailed answers. If you don\'t know something, say so honestly and suggest where the user might find the information.'},
  {name:'Technical Support',prompt:'You are a technical support specialist. Help users troubleshoot issues step by step. Ask clarifying questions, provide clear instructions, and verify the solution works before closing the ticket.'},
  {name:'Onboarding Guide',prompt:'You are a friendly onboarding guide that helps new users get started. Walk them through features, answer questions about how things work, and provide tips for getting the most out of the platform.'},
];
const LANGUAGES=[{code:'',name:'Auto Detect'},{code:'vi',name:'Vietnamese'},{code:'en',name:'English'},{code:'ja',name:'Japanese'},{code:'ko',name:'Korean'},{code:'zh',name:'Chinese'},{code:'th',name:'Thai'},{code:'fr',name:'French'},{code:'de',name:'German'},{code:'es',name:'Spanish'},{code:'pt',name:'Portuguese'}];
let _agentSkills=null,_agentHands=null;
const ARCHETYPES=[{v:'assistant',l:'🤖 Assistant'},{v:'support',l:'📞 Support'},{v:'researcher',l:'🔬 Researcher'},{v:'coder',l:'💻 Coder'},{v:'writer',l:'✍️ Writer'},{v:'analyst',l:'📊 Analyst'},{v:'devops',l:'⚙️ DevOps'}];
const VIBES=[{v:'professional',l:'Professional'},{v:'friendly',l:'Friendly'},{v:'technical',l:'Technical'},{v:'creative',l:'Creative'},{v:'concise',l:'Concise'},{v:'mentor',l:'Mentor'}];
const GREETINGS=[{v:'warm',l:'Warm'},{v:'formal',l:'Formal'},{v:'playful',l:'Playful'},{v:'brief',l:'Brief'}];
const PROFILES=[{v:'full',l:'🌐 Full — All tools'},{v:'coding',l:'💻 Coding — file, shell, web'},{v:'research',l:'🔬 Research — web, file'},{v:'messaging',l:'📨 Messaging — agent, memory'},{v:'minimal',l:'🔒 Minimal — read-only'}];
async function renderAgent(t,canEdit){
  const dis=canEdit?'':'disabled';
  if(!_agentSkills){try{const d=await api('GET','/api/portal/system/skills');_agentSkills=d.skills||[]}catch(e){_agentSkills=[]}}
  if(!_agentHands){try{const d=await api('GET','/api/portal/system/hands');_agentHands=d.hands||[]}catch(e){_agentHands=[]}}
  const curSkills=t.skills||[];const curHands=t.hands||[];
  // Section 1: Agent Identity
  const archOpts=ARCHETYPES.map(a=>`<option value="${a.v}"${(t.archetype||'assistant')===a.v?' selected':''}>${a.l}</option>`).join('');
  const vibeOpts=VIBES.map(v=>`<option value="${v.v}"${(t.vibe||'professional')===v.v?' selected':''}>${v.l}</option>`).join('');
  const greetOpts=GREETINGS.map(g=>`<option value="${g.v}"${(t.greeting_style||'warm')===g.v?' selected':''}>${g.l}</option>`).join('');
  let html=`<div class="config-section" style="border-left:3px solid var(--o);padding-left:16px"><h3 style="margin-bottom:12px">🎭 Agent Identity</h3>
    <div class="config-row"><div class="fg"><label>Agent Name</label><input type="text" id="agentName" value="${t.agent_name||t.name+' Agent'}" placeholder="My AI Agent" ${dis}></div>
    <div class="fg"><label>Archetype</label><select id="agentArchetype" ${dis}>${archOpts}</select></div></div>
    <div class="config-row"><div class="fg"><label>Personality</label><select id="agentVibe" ${dis}>${vibeOpts}</select></div>
    <div class="fg"><label>Greeting Style</label><select id="agentGreeting" ${dis}>${greetOpts}</select></div></div></div>`;
  // Section 2: Model Info (read-only from Config tab)
  html+=`<div class="config-section"><h3 style="margin-bottom:12px">⚙️ Model Config</h3>
    <div class="config-row"><div class="fg"><label>Provider</label><input type="text" value="${t.provider||'groq'}" disabled style="opacity:.7"></div>
    <div class="fg"><label>Model</label><input type="text" value="${t.model||'llama-3.3-70b-versatile'}" disabled style="opacity:.7"></div>
    <div class="fg"><label>Temperature</label><input type="text" value="${t.temperature||0.7}" disabled style="opacity:.7"></div></div>
    <p style="font-size:.75rem;color:var(--d);margin-top:6px">Model settings are configured in the <b>Config</b> tab.</p></div>`;
  // Section 3: System Prompt with Templates
  const tplOpts=PROMPT_TEMPLATES.map(tp=>`<option value="${tp.name}">${tp.name}</option>`).join('');
  html+=`<div class="config-section"><div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:12px"><h3>💬 System Prompt</h3><div style="display:flex;gap:8px;align-items:center"><select id="promptTemplate" onchange="applyTemplate()" ${dis} style="font-size:.8rem;padding:4px 8px;border:1px solid var(--b);border-radius:6px"><option value="">Load Template...</option>${tplOpts}</select><span class="badge plan">${(t.system_prompt||'').length} chars</span></div></div>
    <textarea id="agentPrompt" rows="8" style="width:100%;padding:12px;border:1px solid var(--b);border-radius:8px;font-family:'Inter',sans-serif;font-size:.85rem;resize:vertical;background:var(--bg)" placeholder="You are a helpful customer support agent..." ${dis}>${t.system_prompt||''}</textarea>
    <p style="font-size:.75rem;color:var(--d);margin-top:6px">Define your Agent's personality, rules, and knowledge. This prompt is sent at the start of every conversation.</p></div>`;
  // Section 4: Tool Profile
  const profOpts=PROFILES.map(p=>`<option value="${p.v}"${(t.tool_profile||'full')===p.v?' selected':''}>${p.l}</option>`).join('');
  html+=`<div class="config-section"><h3 style="margin-bottom:12px">🛠️ Tool Profile</h3>
    <select id="agentProfile" ${dis} style="width:100%;padding:10px 12px;border:1px solid var(--b);border-radius:8px;font-size:.85rem;background:var(--bg)">${profOpts}</select>
    <p style="font-size:.75rem;color:var(--d);margin-top:6px">Determines which tools the agent can use. <b>Full</b> = all tools. <b>Minimal</b> = read-only.</p></div>`;
  // Section 5: Skills
  const skillCats={};
  _agentSkills.forEach(s=>{const cat=s.tags&&s.tags[0]?s.tags[0]:'other';if(!skillCats[cat])skillCats[cat]=[];skillCats[cat].push(s)});
  html+=`<div class="config-section"><div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:12px"><h3>📚 Skills</h3><span class="badge running">${curSkills.length} active</span></div>
    <p style="font-size:.8rem;color:var(--d);margin-bottom:12px">Select specialized knowledge areas for your Agent.</p>`;
  Object.keys(skillCats).sort().forEach(cat=>{
    const skills=skillCats[cat];
    html+=`<div style="margin-bottom:12px"><div style="font-size:.75rem;font-weight:600;color:var(--d);text-transform:uppercase;margin-bottom:6px">${cat} (${skills.length})</div><div style="display:flex;flex-wrap:wrap;gap:6px">`;
    skills.forEach(s=>{
      const active=curSkills.includes(s.name);
      const cls=active?'background:var(--o);color:#fff;border-color:var(--o)':'background:var(--bg2);color:var(--t);border-color:var(--b)';
      html+=`<div class="skill-chip" data-skill="${s.name}" onclick="${canEdit?"toggleSkill('"+s.name+"',this)":''}" style="padding:4px 10px;border:1px solid;border-radius:16px;font-size:.75rem;cursor:${canEdit?'pointer':'default'};transition:all .2s;${cls}" title="${s.description||s.name}">${s.name}</div>`;
    });html+=`</div></div>`;
  });html+=`</div>`;
  // Section 6: Hands
  const handIcons={'browser':'&#127760;','researcher':'&#128270;','collector':'&#128203;','lead':'&#128188;','predictor':'&#128200;','twitter':'&#128038;'};
  html+=`<div class="config-section"><div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:12px"><h3>🦾 Hands</h3><span class="badge running">${curHands.length} active</span></div>
    <p style="font-size:.8rem;color:var(--d);margin-bottom:12px">Enable action capabilities for your Agent.</p>
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:12px">`;
  _agentHands.forEach(h=>{
    const active=curHands.includes(h.name);
    const border=active?'border-color:var(--o);background:var(--ol)':'border-color:var(--b);background:var(--bg)';
    const icon=handIcons[h.name]||'&#129302;';
    html+=`<div class="hand-card" data-hand="${h.name}" onclick="${canEdit?"toggleHand('"+h.name+"',this)":''}" style="padding:16px;border:2px solid;border-radius:12px;cursor:${canEdit?'pointer':'default'};transition:all .2s;${border}">
      <div style="font-size:1.5rem;margin-bottom:6px">${icon}</div>
      <div style="font-weight:600;font-size:.85rem;margin-bottom:4px">${h.name.charAt(0).toUpperCase()+h.name.slice(1)}</div>
      <div style="font-size:.75rem;color:var(--d)">${h.description||''}</div>
      <div style="margin-top:8px"><span class="badge ${active?'running':'stopped'}" style="font-size:.65rem">${active?'Active':'Inactive'}</span></div>
    </div>`;
  });html+=`</div></div>`;
  // Section 7: Settings
  const langOpts=LANGUAGES.map(l=>`<option value="${l.code}"${(t.language||'')===l.code?' selected':''}>${l.name}</option>`).join('');
  html+=`<div class="config-section"><h3 style="margin-bottom:12px">⚙️ Settings</h3>
    <div class="config-row"><div class="fg"><label>Primary Language</label><select id="agentLang" ${dis}>${langOpts}</select></div>
    <div class="fg"><label>Webhook URL</label><input type="url" id="agentWebhook" value="${t.webhook_url||''}" placeholder="https://your-crm.com/webhook" ${dis}></div></div>
    <p style="font-size:.75rem;color:var(--d);margin-top:6px">Language sets the default reply language. Webhook receives POST notifications for new messages.</p></div>`;
  // Action buttons
  if(canEdit){html+=`<div style="margin-top:20px;display:flex;gap:10px;align-items:center"><button class="btn-o" onclick="saveAgentConfig()" style="padding:8px 20px">Save Config</button><button onclick="deployAgent()" style="padding:8px 20px;background:linear-gradient(135deg,#e74c3c,#c0392b);color:#fff;border:none;border-radius:8px;cursor:pointer;font-weight:600;font-size:.85rem">🚀 Deploy Agent</button><span id="deployStatus" style="font-size:.8rem;color:var(--d)"></span></div>`}
  return html;
}
function applyTemplate(){const sel=document.getElementById('promptTemplate');const tpl=PROMPT_TEMPLATES.find(t=>t.name===sel.value);if(tpl&&tpl.prompt){document.getElementById('agentPrompt').value=tpl.prompt}}
function toggleSkill(name,el){
  const t=D;if(!t)return;
  const idx=(t.skills||[]).indexOf(name);
  if(idx>=0){t.skills.splice(idx,1);el.style.background='var(--bg2)';el.style.color='var(--t)';el.style.borderColor='var(--b)'}
  else{if(!t.skills)t.skills=[];t.skills.push(name);el.style.background='var(--o)';el.style.color='#fff';el.style.borderColor='var(--o)'}
}
function toggleHand(name,el){
  const t=D;if(!t)return;
  const idx=(t.hands||[]).indexOf(name);
  if(idx>=0){t.hands.splice(idx,1);el.style.borderColor='var(--b)';el.style.background='var(--bg)';el.querySelector('.badge').className='badge stopped';el.querySelector('.badge').textContent='Inactive'}
  else{if(!t.hands)t.hands=[];t.hands.push(name);el.style.borderColor='var(--o)';el.style.background='var(--ol)';el.querySelector('.badge').className='badge running';el.querySelector('.badge').textContent='Active'}
}
function getAgentBody(){
  return {system_prompt:document.getElementById('agentPrompt').value,skills:D.skills||[],hands:D.hands||[],language:document.getElementById('agentLang').value,webhook_url:document.getElementById('agentWebhook').value,agent_name:document.getElementById('agentName').value,archetype:document.getElementById('agentArchetype').value,vibe:document.getElementById('agentVibe').value,greeting_style:document.getElementById('agentGreeting').value,tool_profile:document.getElementById('agentProfile').value};
}
async function saveAgentConfig(){
  if(!D)return;
  const body=getAgentBody();
  const d=await api('PUT','/api/portal/tenants/'+D.id+'/agent',body);
  if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);alert('Agent config saved!')}else{alert(d.error||'Failed')}
}
async function deployAgent(){
  if(!D)return;
  const st=document.getElementById('deployStatus');
  st.innerHTML='<span style="color:var(--o)">Deploying...</span>';
  const body=getAgentBody();body.deploy=true;
  const d=await api('PUT','/api/portal/tenants/'+D.id+'/agent',body);
  if(d.deployed){
    st.innerHTML='<span style="color:#27ae60">✅ Deployed: '+d.agent_name+'</span>';
    D=await api('GET','/api/portal/tenants/'+D.id);
  }else{
    st.innerHTML='<span style="color:#e74c3c">❌ '+(d.deploy_error||'Deploy failed')+'</span>';
  }
}
let chatHistory={};
function renderAssistant(t){
  if(!chatHistory[t.id])chatHistory[t.id]=[];
  const msgs=chatHistory[t.id];
  const msgsHtml=msgs.map(m=>`<div style="display:flex;justify-content:${m.role==='user'?'flex-end':'flex-start'};margin-bottom:8px"><div style="max-width:80%;padding:10px 14px;border-radius:${m.role==='user'?'16px 16px 4px 16px':'16px 16px 16px 4px'};background:${m.role==='user'?'var(--o)':'var(--bg2)'};color:${m.role==='user'?'#fff':'var(--t)'};font-size:.85rem;line-height:1.5;word-break:break-word;white-space:pre-wrap">${escHtml(m.text)}</div></div>`).join('');
  return `<div class="config-section" style="padding:0;display:flex;flex-direction:column;height:60vh;min-height:400px">
    <div style="padding:12px 16px;border-bottom:1px solid var(--b);display:flex;align-items:center;justify-content:space-between">
      <div><b>💬 ${t.agent_name||t.name+' Agent'}</b><span style="font-size:.75rem;color:var(--d);margin-left:8px">${t.provider}/${t.model}</span></div>
      <button class="btn-g" onclick="chatHistory['${t.id}']=[]; renderDetailBody()" style="font-size:.75rem;padding:4px 10px">Clear</button>
    </div>
    <div id="chatMsgs" style="flex:1;overflow-y:auto;padding:16px;display:flex;flex-direction:column">${msgsHtml||'<div style="text-align:center;color:var(--d);margin:auto;font-size:.85rem">Send a message to start chatting with your agent.<br>Make sure to <b>Deploy</b> first in the Agent tab.</div>'}</div>
    <div style="padding:12px 16px;border-top:1px solid var(--b);display:flex;gap:8px">
      <input type="text" id="chatInput" placeholder="Type a message..." onkeydown="if(event.key==='Enter'&&!event.shiftKey){event.preventDefault();sendChatMsg()}" style="flex:1;padding:10px 14px;border:1px solid var(--b);border-radius:20px;font-size:.85rem;outline:none;background:var(--bg)">
      <button onclick="sendChatMsg()" style="padding:10px 20px;background:var(--o);color:#fff;border:none;border-radius:20px;cursor:pointer;font-weight:600;font-size:.85rem;white-space:nowrap">Send</button>
    </div>
  </div>`;
}
function escHtml(s){return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')}
async function sendChatMsg(){
  if(!D)return;
  const inp=document.getElementById('chatInput');
  const msg=inp.value.trim();if(!msg)return;
  if(!chatHistory[D.id])chatHistory[D.id]=[];
  chatHistory[D.id].push({role:'user',text:msg});
  inp.value='';renderDetailBody();
  const el=document.getElementById('chatMsgs');if(el)el.scrollTop=el.scrollHeight;
  // Show typing indicator
  chatHistory[D.id].push({role:'assistant',text:'...'});renderDetailBody();
  if(el)el.scrollTop=el.scrollHeight;
  try{
    const d=await api('POST','/api/portal/tenants/'+D.id+'/chat',{message:msg});
    chatHistory[D.id].pop();// remove typing
    if(d.error){chatHistory[D.id].push({role:'assistant',text:'❌ '+d.error})}
    else{chatHistory[D.id].push({role:'assistant',text:d.response||'(no response)'})}
  }catch(e){chatHistory[D.id].pop();chatHistory[D.id].push({role:'assistant',text:'❌ Connection error'})}
  renderDetailBody();
  const el2=document.getElementById('chatMsgs');if(el2)el2.scrollTop=el2.scrollHeight;
}
async function cloneTenant(){
  if(!D)return;if(!confirm('Clone tenant "'+D.name+'"? A new copy will be created.'))return;
  const d=await api('POST','/api/portal/tenants/'+D.id+'/clone');
  if(d.ok){alert('Tenant cloned! New ID: '+d.tenant_id);await loadT();showPage('tenants')}else{alert(d.error||'Failed')}
}

// Tab: History (Conversation History)
async function renderHistory(t){
  let html=`<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px"><h3 style="font-size:1rem;font-weight:700">Conversation History</h3></div>`;
  try{
    const d=await api('GET','/api/portal/tenants/'+t.id+'/conversations');
    if(d.error&&(!d.messages||d.messages.length===0)){
      html+=`<div class="empty"><div class="empty-icon">&#128172;</div><h4>${d.error||'No conversations yet'}</h4><p>Deploy your agent first, then conversations will appear here.</p></div>`;
    }else{
      const msgs=d.messages||[];
      if(msgs.length===0){
        html+=`<div class="empty"><div class="empty-icon">&#128172;</div><h4>No messages yet</h4><p>Start chatting with your agent in the Assistant tab or via a connected channel.</p></div>`;
      }else{
        html+=`<div class="sr"><span class="sl">Messages: <span class="sv">${msgs.length}</span></span><span class="sl">Session: <span class="sv" style="font-size:.7rem">${d.session_id||'-'}</span></span></div>`;
        html+=`<div style="max-height:500px;overflow-y:auto;border:1px solid var(--b);border-radius:10px;padding:12px;background:var(--bg)">`;
        msgs.forEach(m=>{
          const isUser=m.role==='User';
          const roleColor=isUser?'var(--o)':'#27ae60';
          const roleName=isUser?'You':'Agent';
          const content=(m.content||'').replace(/</g,'&lt;').replace(/>/g,'&gt;');
          html+=`<div style="margin-bottom:12px;display:flex;gap:8px;flex-direction:${isUser?'row-reverse':'row'}">
            <div style="flex-shrink:0;width:28px;height:28px;border-radius:50%;background:${roleColor};color:#fff;display:flex;align-items:center;justify-content:center;font-size:.7rem;font-weight:700">${isUser?'U':'A'}</div>
            <div style="max-width:80%;padding:8px 12px;border-radius:12px;background:${isUser?'var(--ol)':'var(--bg2)'};font-size:.83rem;line-height:1.5;white-space:pre-wrap;word-break:break-word">${content}</div>
          </div>`;
          if(m.tools&&m.tools.length>0){
            m.tools.forEach(tool=>{
              html+=`<div style="margin:0 0 8px 36px;padding:6px 10px;background:var(--bg2);border-left:3px solid var(--o);border-radius:4px;font-size:.75rem"><b>🔧 ${tool.name}</b>${tool.result?' → <span style="color:var(--d)">'+tool.result.substring(0,200)+'</span>':''}</div>`;
            });
          }
        });
        html+=`</div>`;
      }
    }
  }catch(e){
    html+=`<div class="empty"><div class="empty-icon">&#9888;</div><h4>Could not load conversations</h4><p>${e.message||'Error'}</p></div>`;
  }
  return html;
}

// Tab: Usage
function renderUsage(t){
  const pct=t.max_messages_per_day>=INF?0:Math.min(100,Math.round(t.messages_today/t.max_messages_per_day*100));
  return `<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:20px"><h3 style="font-size:1rem;font-weight:700">Usage</h3><span class="badge plan" style="padding:6px 12px;font-size:.8rem">Current Period</span></div>
  <div class="cards" style="grid-template-columns:repeat(3,1fr)">
    <div class="card"><div class="card-label">Messages Today</div><div class="card-val" style="font-size:2rem">${t.messages_today}</div></div>
    <div class="card"><div class="card-label">Daily Limit</div><div class="card-val" style="font-size:2rem">${fmt(t.max_messages_per_day)}</div></div>
    <div class="card"><div class="card-label">Members</div><div class="card-val" style="font-size:2rem">${(t.members||[]).length} / ${fmt(t.max_members)}</div></div>
  </div>
  <div class="sbox"><h3>Daily Message Usage</h3>
    <div class="bar" style="height:20px;margin-top:12px;background:var(--bg3);border-radius:8px;overflow:hidden"><div style="height:100%;background:${pct>80?'var(--r)':pct>50?'#f59e0b':'var(--g)'};border-radius:8px;width:${pct}%;transition:width .3s"></div></div>
    <p style="font-size:.8rem;color:var(--d);margin-top:8px">${t.max_messages_per_day>=INF?'Unlimited':pct+'% used ('+t.messages_today+' / '+t.max_messages_per_day+')'}</p>
  </div>`;
}

// Tab: Members
function renderMembersTab(t,isAdmin){
  const isOwner=(t.members||[]).some(m=>m.email.toLowerCase()===S.email.toLowerCase()&&(m.role==='owner'||m.role==='manager'));
  const canEdit=isAdmin||isOwner;
  const addBtn=canEdit?`<button class="btn-o" onclick="openModal('addMemberModal')">+ Add Member</button>`:'';
  const header=`<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px"><h3 style="font-size:1rem;font-weight:700">Members</h3>${addBtn}</div>`;
  const rows=(t.members||[]).map(m=>{
    const roleHtml=canEdit?`<select class="role-sel" onchange="changeRole('${m.email}',this.value)">${ROLES.map(r=>`<option value="${r.toLowerCase()}"${m.role===r.toLowerCase()?' selected':''}>${r}</option>`).join('')}</select>`:`<span class="badge plan">${m.role.charAt(0).toUpperCase()+m.role.slice(1)}</span>`;
    const actions=canEdit?`<button class="btn-r" onclick="removeMember('${m.email}')">Remove</button>`:'';
    return `<tr><td style="font-weight:500">${m.email}</td><td>${roleHtml}</td><td style="color:var(--d)">${fmtDate(m.added_at)}</td><td>${actions}</td></tr>`;
  }).join('');
  return header+`<table class="dt"><thead><tr><th>Email</th><th>Role</th><th>Joined</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}

// Tenant Actions
async function doRestart(){if(!D)return;const d=await api('POST',`/api/portal/tenants/${D.id}/restart`);if(d.ok){await loadT();D=await api('GET','/api/portal/tenants/'+D.id);renderDetailPage()}else{alert(d.error||'Failed')}}
async function doStop(){if(!D||!confirm('Stop this tenant?'))return;const d=await api('POST',`/api/portal/tenants/${D.id}/stop`);if(d.ok){await loadT();D=await api('GET','/api/portal/tenants/'+D.id);renderDetailPage()}else{alert(d.error||'Failed')}}
async function doDeleteTenant(){if(!D||!confirm('Delete tenant "'+D.name+'"? This cannot be undone.'))return;const d=await api('DELETE','/api/portal/tenants/'+D.id);if(d.ok){await loadT();showPage('tenants')}else{alert(d.error||'Failed')}}
async function doDeleteFromList(id,name){if(!confirm('Delete tenant "'+name+'"? This cannot be undone.'))return;const d=await api('DELETE','/api/portal/tenants/'+id);if(d.ok){await loadT();renderList()}else{alert(d.error||'Failed')}}

// Config Actions
async function saveConfig(){if(!D)return;const body={provider:document.getElementById('cfgProvider').value,model:document.getElementById('cfgModel').value,temperature:parseFloat(document.getElementById('cfgTemp').value)||0.7};const key=document.getElementById('cfgApiKey').value.trim();if(key)body.api_key=key;const d=await api('PUT',`/api/portal/tenants/${D.id}/config`,body);if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody();alert('Config saved!')}else{alert(d.error||'Failed')}}

// Channel Actions
async function addChannel(){const ct=document.getElementById('acType').value,nm=document.getElementById('acName').value.trim();if(!ct){alert('Channel type is required');return}const body={channel_type:ct};if(nm)body.name=nm;const d=await api('POST',`/api/portal/tenants/${D.id}/channels`,body);if(d.ok){closeModal('addChannelModal');document.getElementById('acName').value='';D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}
async function removeChannel(name){if(!confirm('Remove channel "'+name+'"?'))return;const d=await api('DELETE',`/api/portal/tenants/${D.id}/channels`,{name});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}

// Member Actions
async function changeRole(email,role){const d=await api('PUT',`/api/portal/tenants/${D.id}/members/role`,{email,role});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}
async function removeMember(email){if(!confirm('Remove '+email+'?'))return;const d=await api('DELETE',`/api/portal/tenants/${D.id}/members`,{email});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}
function openModal(id){document.getElementById(id).classList.add('show')}
function closeModal(id){document.getElementById(id).classList.remove('show')}
async function doAddMember(){const e=document.getElementById('amEmail').value.trim(),n=document.getElementById('amName').value.trim(),r=document.getElementById('amRole').value,p=document.getElementById('amPass').value;if(!e){alert('Email is required');return}const body={email:e,role:r};if(n)body.display_name=n;if(p)body.password=p;const d=await api('POST',`/api/portal/tenants/${D.id}/members`,body);if(d.ok){closeModal('addMemberModal');document.getElementById('amEmail').value='';document.getElementById('amName').value='';document.getElementById('amPass').value='';D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}

// All Members Page
async function renderMembers(){
  const d=await api('GET','/api/portal/members');const ms=d.members||[];
  const rows=ms.map(m=>`<tr><td style="font-weight:500">${m.display_name||m.email}</td><td style="color:var(--d)">${m.email}</td><td><span class="badge plan">${m.role}</span></td><td>${m.has_password?'Yes':'No'}</td><td>${(m.tenants||[]).map(t=>t.name).join(', ')||'-'}</td><td style="color:var(--d);font-size:.8rem">${m.last_login||'Never'}</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Members: <span class="sv">${ms.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Password</th><th>Tenants</th><th>Last Login</th></tr></thead><tbody>${rows}</tbody></table>`;
}

// Users Page
let editingUser=null;
async function renderUsers(){
  const d=await api('GET','/api/portal/users');const us=d.users||[];
  const pd=await api('GET','/api/portal/plans');const plans=pd.plans||[];
  const rows=us.map(u=>{
    if(editingUser===u.email){
      const planOpts=plans.map(p=>`<option value="${p.id}"${u.plan_id===p.id?' selected':''}>${p.name}</option>`).join('');
      return `<tr style="background:var(--ol)"><td colspan="8"><div style="display:flex;flex-wrap:wrap;gap:8px;align-items:center;padding:8px 0"><strong>${u.email}</strong><select id="euRole"><option value="user"${u.role==='user'?' selected':''}>User</option><option value="admin"${u.role==='admin'?' selected':''}>Admin</option></select><select id="euPlan">${planOpts}</select><input type="password" id="euPass" placeholder="New password (optional)" style="width:160px"><button class="btn-o" onclick="saveEditUser('${u.email}')">Save</button><button class="btn-cancel" onclick="editingUser=null;renderUsers()">Cancel</button></div></td></tr>`;
    }
    return `<tr><td style="font-weight:500">${u.display_name||u.email}</td><td style="color:var(--d)">${u.email}</td><td><span class="badge ${u.role==='admin'?'running':'plan'}">${u.role}</span></td><td><span class="badge plan">${u.plan_id||'none'}</span></td><td>${u.tenant_count||0} / ${fmt(u.max_tenants)}</td><td>${u.has_password?'Yes':'No'}</td><td style="color:var(--d);font-size:.8rem">${u.last_login?fmtDate(u.last_login):'Never'}</td><td><button class="btn-g" onclick="editingUser='${u.email}';renderUsers()">Edit</button> <button class="btn-r" onclick="deleteUser('${u.email}',${u.tenant_count||0})">Delete</button></td></tr>`;
  }).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Users: <span class="sv">${us.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Plan</th><th>Tenants</th><th>Password</th><th>Last Login</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
async function openCreateUserModal(){const d=await api('GET','/api/portal/plans');const plans=d.plans||[];const sel=document.getElementById('cuPlan');sel.innerHTML=plans.map(p=>`<option value="${p.id}"${p.is_default?' selected':''}>${p.name} (${p.price_label||'Free'})</option>`).join('');openModal('createUserModal')}
async function doCreateUser(){const email=document.getElementById('cuEmail').value.trim(),name=document.getElementById('cuName').value.trim(),pass=document.getElementById('cuPass').value,role=document.getElementById('cuRole').value,plan=document.getElementById('cuPlan').value;if(!email){alert('Email is required');return}const body={email,role,plan_id:plan};if(name)body.display_name=name;if(pass)body.password=pass;const d=await api('POST','/api/portal/users',body);if(d.ok){closeModal('createUserModal');document.getElementById('cuEmail').value='';document.getElementById('cuName').value='';document.getElementById('cuPass').value='';renderUsers()}else{alert(d.error||'Failed')}}
async function saveEditUser(email){const body={email,role:document.getElementById('euRole').value,plan_id:document.getElementById('euPlan').value};const pw=document.getElementById('euPass').value;if(pw)body.password=pw;const d=await api('PUT','/api/portal/users/'+encodeURIComponent(email),body);if(d.ok){editingUser=null;renderUsers()}else{alert(d.error||'Failed')}}
async function deleteUser(email,tenantCount){const msg=tenantCount>0?`Delete user "${email}"?\n\nThis user is a member of ${tenantCount} tenant(s). They will be removed from all tenants.`:`Delete user "${email}"?`;if(!confirm(msg))return;const d=await api('DELETE','/api/portal/users/'+encodeURIComponent(email));if(d.ok){if(d.removed_from_tenants&&d.removed_from_tenants.length>0)alert('User removed from tenants: '+d.removed_from_tenants.join(', '));renderUsers()}else alert(d.error||'Failed')}

// Plans Page
let editingPlan=null;
async function renderPlans(){
  const d=await api('GET','/api/portal/plans');const ps=d.plans||[];
  const rows=ps.map(p=>{
    if(editingPlan===p.id){
      return `<tr style="background:var(--ol)"><td colspan="7"><div style="display:flex;flex-wrap:wrap;gap:8px;align-items:center;padding:8px 0"><input type="text" id="epName" value="${p.name}" style="width:120px"><input type="number" id="epMsg" value="${p.max_messages_per_day}" style="width:80px" placeholder="Msg/Day"><input type="number" id="epCh" value="${p.max_channels}" style="width:60px" placeholder="Ch"><input type="number" id="epMem" value="${p.max_members}" style="width:60px" placeholder="Mem"><input type="number" id="epTen" value="${p.max_tenants}" style="width:60px" placeholder="Ten"><input type="text" id="epPrice" value="${p.price_label||''}" style="width:80px" placeholder="Price"><button class="btn-o" onclick="saveEditPlan('${p.id}')">Save</button><button class="btn-cancel" onclick="editingPlan=null;renderPlans()">Cancel</button></div></td></tr>`;
    }
    return `<tr><td style="font-weight:500">${p.name}${p.is_default?' <span class="badge running" style="font-size:.7rem">Default</span>':''}</td><td>${fmt(p.max_messages_per_day)}</td><td>${fmt(p.max_channels)}</td><td>${fmt(p.max_members)}</td><td>${fmt(p.max_tenants)}</td><td>${p.price_label||'-'}</td><td><button class="btn-g" onclick="editingPlan='${p.id}';renderPlans()">Edit</button> <button class="btn-r" onclick="deletePlan('${p.id}')">Delete</button></td></tr>`;
  }).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Plans: <span class="sv">${ps.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Msg/Day</th><th>Channels</th><th>Members</th><th>Tenants</th><th>Price</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
async function doCreatePlan(){const name=document.getElementById('cpName').value.trim();if(!name){alert('Plan name is required');return}const body={name,max_messages_per_day:parseInt(document.getElementById('cpMsg').value)||500,max_channels:parseInt(document.getElementById('cpCh').value)||5,max_members:parseInt(document.getElementById('cpMem').value)||10,max_tenants:parseInt(document.getElementById('cpTen').value)||5,price_label:document.getElementById('cpPrice').value.trim()};const d=await api('POST','/api/portal/plans',body);if(d.ok){closeModal('createPlanModal');document.getElementById('cpName').value='';renderPlans()}else{alert(d.error||'Failed')}}
async function saveEditPlan(id){const body={name:document.getElementById('epName').value.trim(),max_messages_per_day:parseInt(document.getElementById('epMsg').value)||500,max_channels:parseInt(document.getElementById('epCh').value)||5,max_members:parseInt(document.getElementById('epMem').value)||10,max_tenants:parseInt(document.getElementById('epTen').value)||5,price_label:document.getElementById('epPrice').value.trim()};const d=await api('PUT','/api/portal/plans/'+encodeURIComponent(id),body);if(d.ok){editingPlan=null;renderPlans()}else{alert(d.error||'Failed')}}
async function deletePlan(id){if(!confirm('Delete plan "'+id+'"?'))return;const d=await api('DELETE','/api/portal/plans/'+encodeURIComponent(id));if(d.ok)renderPlans();else alert(d.error||'Failed')}

// Create Tenant (self-service)
function openCreateTenantModal(){openModal('createTenantModal')}
async function doCreateMyTenant(){const name=document.getElementById('ctName').value.trim();if(!name){alert('Tenant name is required');return}const body={name,provider:document.getElementById('ctProvider').value,model:document.getElementById('ctModel').value};const d=await api('POST','/api/portal/my/tenants',body);if(d.ok){closeModal('createTenantModal');document.getElementById('ctName').value='';await loadT();showPage('tenants')}else{alert(d.error||'Failed')}}

// Init + Permalink
window.addEventListener('popstate',function(e){if(e.state&&e.state.page==='detail'&&e.state.id){openDetail(e.state.id)}else{showPage('tenants')}});
(function(){const s=localStorage.getItem('ps');if(s){try{S=JSON.parse(s);
  // Check if URL has tenant ID (permalink)
  const m=location.pathname.match(/\/([a-f0-9-]{36})/i);
  if(m){showDash().then(()=>openDetail(m[1]))}else{showDash()}
}catch(e){localStorage.removeItem('ps')}}})();
</script>
</body></html>"##;

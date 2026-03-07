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
:root{--o:#FF5C00;--oh:#e65200;--obg:rgba(255,92,0,.08);--ol:#fff7ed;--bg:#fff;--bg2:#F9F8F7;--bg3:#EDECEB;--t:#1A1817;--d:#6B6560;--m:#9ca3af;--b:#D5D2CF;--g:#22c55e;--gb:#f0fdf4;--gt:#15803d;--r:#ef4444;--rb:#fef2f2;--rt:#dc2626;--pb:#faf5ff;--pt:#7c3aed;--bb:#eff6ff;--bt:#2563eb;--sbbg:#EDECEB}
body{font-family:'Inter',system-ui,sans-serif;margin:0;min-height:100vh;background:var(--bg2)}
/* Login Screen */
@keyframes blink{0%,100%{opacity:1}50%{opacity:0}}
@keyframes fadeInUp{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}
@keyframes pulseGlow{0%,100%{opacity:.4}50%{opacity:.7}}
@keyframes floatDot{0%,100%{transform:translateY(0)}50%{transform:translateY(-6px)}}
.login-screen{display:flex;min-height:100vh;background:var(--bg)}
.login-left{flex:1;background:#0f172a;position:relative;display:flex;flex-direction:column;justify-content:center;padding:48px 64px;overflow:hidden;color:#e2e8f0}
.login-left::before{content:'';position:absolute;inset:0;background-image:radial-gradient(circle at 1px 1px,rgba(255,255,255,.06) 1px,transparent 0);background-size:32px 32px}
.login-left::after{content:'';position:absolute;top:-120px;right:-120px;width:400px;height:400px;background:radial-gradient(circle,rgba(255,92,0,.15),transparent 70%);animation:pulseGlow 4s ease-in-out infinite;pointer-events:none}
.login-left>*{position:relative;z-index:1}
.brand{display:flex;align-items:center;gap:10px;margin-bottom:40px}
.brand svg{width:36px;height:36px}.brand span{font-size:1.4rem;font-weight:700;letter-spacing:-.5px;color:#fff}
.login-left h2{font-size:2.2rem;font-weight:700;line-height:1.2;letter-spacing:-1px;margin-bottom:16px;color:#fff}
.hl{color:var(--o)}
.login-left .desc{color:#94a3b8;font-size:.95rem;line-height:1.6;margin-bottom:40px}
.tc{background:#1e293b;border:1px solid rgba(255,255,255,.1);border-radius:12px;overflow:hidden;box-shadow:0 8px 32px rgba(0,0,0,.3);margin-bottom:40px}
.td{display:flex;gap:6px;padding:12px 16px;border-bottom:1px solid rgba(255,255,255,.08);align-items:center}
.td span{width:10px;height:10px;border-radius:50%}
.td span:nth-child(1){background:#ff5f57}.td span:nth-child(2){background:#febc2e}.td span:nth-child(3){background:#28c840}
.td-title{margin-left:12px;font-size:.7rem;color:#64748b;font-family:'JetBrains Mono',monospace}
.tcd{padding:16px 20px;font-family:'JetBrains Mono',monospace;font-size:.78rem;line-height:2;color:#64748b;min-height:180px}
.tcd .line{opacity:0;white-space:nowrap;overflow:hidden}
.tcd .line.visible{opacity:1;animation:fadeInUp .3s ease}
.tcd .prompt{color:#a78bfa}.tcd .cmd{color:#22d3ee}.tcd .ok{color:#4ade80}.tcd .warn{color:#fbbf24}.tcd .info{color:#94a3b8}
.tcd .cursor{display:inline-block;width:7px;height:14px;background:var(--o);animation:blink 1s step-end infinite;vertical-align:middle;margin-left:2px;border-radius:1px}
.mets{display:flex;gap:24px}
.met{background:rgba(255,255,255,.05);border:1px solid rgba(255,255,255,.08);border-radius:10px;padding:16px 20px;flex:1;transition:all .3s ease;cursor:default}
.met:hover{background:rgba(255,92,0,.1);border-color:rgba(255,92,0,.3);transform:translateY(-2px)}
.met .v{font-size:1.4rem;font-weight:700;color:#fff}.met .v .u{color:var(--o);font-weight:600}
.met .l{font-size:.7rem;color:#64748b;margin-top:4px;text-transform:uppercase;letter-spacing:.5px}
.login-tags{display:flex;gap:8px;margin-top:32px}
.login-tag{padding:4px 12px;border-radius:20px;font-size:.7rem;font-weight:500;border:1px solid rgba(255,255,255,.1);color:#94a3b8;background:rgba(255,255,255,.03)}
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
.bl{width:100%;padding:14px;background:var(--o);color:#fff;border:none;border-radius:12px;font-size:.95rem;font-weight:600;font-family:inherit;cursor:pointer;transition:background .2s,transform .15s;margin-top:8px}
.bl:hover{background:var(--oh);transform:translateY(-1px)}.bl:disabled{opacity:.5;cursor:not-allowed;transform:none}
.em{color:var(--r);font-size:.8rem;margin-top:12px;display:none}
.lf{margin-top:24px;text-align:center;font-size:.8rem;color:var(--m)}.lf a{color:var(--o);text-decoration:none;font-weight:500}
/* Dashboard Layout */
.dashboard{display:none;min-height:100vh}
.dl{display:flex;min-height:100vh}
.sb{width:240px;background:var(--sbbg);border-right:1px solid var(--b);display:flex;flex-direction:column;flex-shrink:0;position:fixed;left:0;top:0;bottom:0;z-index:10}
.sbh{padding:16px 20px;display:flex;align-items:center;gap:10px;border-bottom:1px solid rgba(0,0,0,.06)}
.sbh svg{width:28px;height:28px}.sbh span{font-size:1rem;font-weight:700;color:var(--o);letter-spacing:1px;text-transform:uppercase;font-family:'JetBrains Mono',monospace}
.sbu{padding:12px 20px;font-size:.8rem;color:var(--d);border-bottom:1px solid rgba(0,0,0,.06)}
.sbn{flex:1;padding:8px;overflow-y:auto;scrollbar-width:thin;scrollbar-color:rgba(0,0,0,.1) transparent}
.si{display:flex;align-items:center;gap:10px;padding:9px 12px;border-radius:8px;font-size:.82rem;font-weight:500;color:var(--d);cursor:pointer;transition:all .15s;text-decoration:none}
.sb-label{padding:18px 12px 6px;font-size:.65rem;font-weight:600;letter-spacing:.1em;text-transform:uppercase;color:var(--m)}
.wf-step{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:14px;margin-bottom:8px;position:relative}.wf-step .step-num{position:absolute;top:10px;left:14px;font-size:.7rem;font-weight:600;color:var(--m)}.wf-step .step-del{position:absolute;top:8px;right:10px;background:none;border:none;color:var(--r);cursor:pointer;font-size:1rem;padding:2px 6px}.wf-step .config-row{display:flex;gap:8px;margin-top:4px}.wf-step .fg{margin-top:6px}.wf-step label{font-size:.75rem;font-weight:500;color:var(--d)}.wf-step input,.wf-step select,.wf-step textarea{width:100%;padding:6px 10px;border:1px solid var(--b);border-radius:6px;font-size:.8rem;background:var(--bg);color:var(--t)}
.si:hover{background:rgba(0,0,0,.04);color:var(--t)}.si.active{background:var(--o);color:#fff;font-weight:600}
.si.active svg{stroke:#fff}
.si svg{width:18px;height:18px;flex-shrink:0}
.sbb{padding:8px;border-top:1px solid rgba(0,0,0,.06)}
.sbb .si{font-size:.8rem;padding:8px 12px}
.mn{flex:1;margin-left:240px;display:flex;flex-direction:column;min-height:100vh;background:var(--bg2)}
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
.dt{width:100%;border-collapse:collapse;font-size:.85rem;background:var(--bg);border:1px solid var(--b);border-radius:12px;overflow:hidden;box-shadow:0 1px 2px rgba(0,0,0,.04)}
.dt th{padding:12px 16px;text-align:left;font-weight:600;font-size:.7rem;text-transform:uppercase;letter-spacing:.05em;color:var(--d);background:var(--bg2);border-bottom:1px solid var(--b)}
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
.sbox{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:20px;margin-bottom:20px;box-shadow:0 1px 2px rgba(0,0,0,.04)}
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
@media(min-width:901px) and (max-width:1200px){.login-left{padding:32px 40px}.login-left h2{font-size:1.8rem}.mets{gap:12px}.met{padding:12px 14px}}
@media(max-width:768px){.tabs{overflow-x:auto;-webkit-overflow-scrolling:touch;flex-wrap:nowrap;gap:0;padding-bottom:4px}.tab{white-space:nowrap;flex-shrink:0;padding:8px 12px;font-size:.8rem}.config-section{padding:12px}.config-row{flex-direction:column;gap:8px}.dh h2{font-size:1.1rem}.dh-meta{font-size:.75rem}.mn{padding:12px}#headerActions{flex-wrap:wrap;gap:6px}#headerActions a,#headerActions button{font-size:.75rem;padding:6px 10px}.dt{font-size:.8rem}.dt th,.dt td{padding:6px 8px}}
</style>
</head>
<body>
<!-- LOGIN -->
<div class="login-screen" id="loginView">
  <div class="login-left">
    <div class="brand"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="var(--o)"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
    <h2>Deploy &amp; manage<br>AI agents with <span class="hl">the official<br>OpenFang runtime</span></h2>
    <p class="desc">Self-service portal for team members. Manage your tenants, view analytics, and collaborate securely.</p>
    <div class="tc"><div class="td"><span></span><span></span><span></span><span class="td-title">terminal — openfang</span></div><div class="tcd" id="terminalBody"></div></div>
    <div class="mets"><div class="met"><div class="v">32 <span class="u">MB</span></div><div class="l">Binary</div></div><div class="met"><div class="v">180<span class="u">ms</span></div><div class="l">Cold Start</div></div><div class="met"><div class="v">26<span class="u">+</span></div><div class="l">Providers</div></div></div>
    <div class="login-tags"><span class="login-tag">🦀 Built with Rust</span><span class="login-tag">⚡ Open Source</span><span class="login-tag">🔒 Self-hosted</span></div>
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
        <div class="sb-label">Channels</div>
        <a class="si" onclick="showPage('channel-instances')" id="channelInstancesNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>Multi Channels</a>
        <div class="sb-label">Agent Features</div>
        <a class="si" onclick="showPage('agents')" id="agentsNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>Agents</a>
        <a class="si" onclick="showPage('knowledge')" id="knowledgeNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/></svg>Kho tri thức</a>
        <a class="si" onclick="showPage('tools')" id="toolsNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"/></svg>Công cụ</a>
        <a class="si" onclick="showPage('skills')" id="skillsNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>Skills Market</a>
        <a class="si" onclick="showPage('gallery')" id="galleryNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg>Gallery</a>
        <div class="sb-label">Automation</div>
        <a class="si" onclick="showPage('workflows')" id="workflowsNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>Workflows</a>
        <a class="si" onclick="showPage('scheduler')" id="schedulerNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>Scheduler</a>
        <a class="si" onclick="showPage('orchestration')" id="orchestrationNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9"/></svg>Điều phối</a>
        <a class="si" onclick="showPage('orgmap')" id="orgmapNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>Org Map</a>
        <a class="si" onclick="showPage('kanban')" id="kanbanNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M8 3v18M16 3v18"/></svg>Kanban</a>
        <div class="sb-label">Monitoring</div>
        <a class="si" onclick="showPage('traces')" id="tracesNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>LLM Traces</a>
        <a class="si" onclick="showPage('cost')" id="costNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/></svg>Cost Tracking</a>
        <a class="si" onclick="showPage('activity')" id="activityNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>Activity Feed</a>
        <a class="si" onclick="showPage('apikeys')" id="apikeysNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>API Keys</a>
        <a class="si" onclick="showPage('usage')" id="usageNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 20V10M12 20V4M6 20v-6"/></svg>Usage & Quotas</a>
        <a class="si" onclick="showPage('configfile')" id="configfileNav"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>Tệp cấu hình</a>
        <div class="sb-label" id="managementLabel" style="display:none">Management</div>
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

<!-- Create Workflow Modal -->
<div class="modal-bg" id="createWorkflowModal">
  <div class="modal" style="max-width:720px;max-height:90vh;overflow-y:auto">
    <h3>Create Workflow</h3>
    <div class="fg"><label>Template</label><select id="wfTemplate" onchange="fillWfTemplate()">
      <option value="custom">Custom Workflow</option>
      <option value="code-review">📝 Code Review Pipeline</option>
      <option value="research-write">🔍 Research &amp; Write Article</option>
      <option value="brainstorm">🧠 Multi-Agent Brainstorm</option>
      <option value="iterative">🔄 Iterative Refinement</option>
    </select></div>
    <div class="config-row"><div class="fg"><label>Name</label><input type="text" id="wfName" placeholder="e.g. my-code-review"></div><div class="fg"><label>Description</label><input type="text" id="wfDesc" placeholder="What this workflow does"></div></div>
    <div style="margin:12px 0 4px"><label style="font-weight:600;font-size:.85rem">Steps</label></div>
    <div id="wfStepsContainer"></div>
    <button class="btn-g" style="width:100%;margin:8px 0" onclick="addWfStep()">+ Add Step</button>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createWorkflowModal')">Cancel</button><button class="btn-o" onclick="doCreateWorkflow()">Create Workflow</button></div>
  </div>
</div>

<!-- Run Workflow Modal -->
<div class="modal-bg" id="runWorkflowModal">
  <div class="modal">
    <h3>Run Workflow</h3>
    <div class="fg"><label>Input Text</label><textarea id="wfRunInput" rows="4" style="width:100%;font-family:monospace;font-size:.85rem;background:var(--bg2);color:var(--t);border:1px solid var(--b);border-radius:8px;padding:10px;resize:vertical" placeholder="Enter the initial input for step 1..."></textarea></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('runWorkflowModal')">Cancel</button><button class="btn-o" id="wfRunBtn" onclick="doRunWorkflow()">Run</button></div>
    <div id="wfRunResult" style="margin-top:12px"></div>
  </div>
</div>

<!-- Create Scheduler Job Modal -->
<div class="modal-bg" id="createSchedulerModal">
  <div class="modal" style="max-width:600px">
    <h3>Create Scheduled Job</h3>
    <div class="fg"><label>Job Name</label><input type="text" id="sjName" placeholder="e.g. daily-report"></div>
    <div class="fg"><label>Cron Expression (5-field: min hour dom mon dow)</label>
      <div class="config-row" style="gap:8px;align-items:center">
        <input type="text" id="sjCron" placeholder="*/5 * * * *" style="flex:1">
        <select id="sjCronPreset" onchange="if(this.value)document.getElementById('sjCron').value=this.value" style="flex:0 0 auto;width:auto">
          <option value="">Preset...</option>
          <option value="* * * * *">Every minute</option>
          <option value="*/5 * * * *">Every 5 min</option>
          <option value="*/15 * * * *">Every 15 min</option>
          <option value="0 * * * *">Every hour</option>
          <option value="0 */6 * * *">Every 6 hours</option>
          <option value="0 9 * * *">Daily at 9 AM</option>
          <option value="0 9 * * 1-5">Weekdays 9 AM</option>
          <option value="0 0 * * 0">Weekly (Sunday)</option>
          <option value="0 0 1 * *">Monthly</option>
        </select>
      </div>
    </div>
    <div class="fg"><label>Target Agent</label><select id="sjAgentId"><option value="">Loading agents...</option></select></div>
    <div class="fg"><label>Message (prompt sent to agent)</label><textarea id="sjMessage" rows="3" style="width:100%;font-family:monospace;font-size:.85rem;background:var(--bg2);color:var(--t);border:1px solid var(--b);border-radius:8px;padding:10px;resize:vertical" placeholder="Generate a daily report of all activities..."></textarea></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createSchedulerModal')">Cancel</button><button class="btn-o" onclick="doCreateSchedulerJob()">Create Job</button></div>
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

<!-- Add Channel Instance Modal -->
<div class="modal-bg" id="addChannelInstanceModal">
  <div class="modal">
    <h3>Add Channel Instance</h3>
    <div class="fg"><label>Tenant</label><select id="ciTenant"></select></div>
    <div class="fg"><label>Channel Type</label><select id="ciType"><option value="telegram">Telegram</option><option value="zalo">Zalo OA</option><option value="discord">Discord</option><option value="slack">Slack</option><option value="whatsapp">WhatsApp</option><option value="facebook">Facebook</option><option value="email">Email</option><option value="web">Web Widget</option></select></div>
    <div class="fg"><label>Display Name</label><input type="text" id="ciName" placeholder="e.g. Shop A - Telegram Bot"></div>
    <div class="fg"><label>Bot Token</label><input type="text" id="ciToken" placeholder="e.g. 123456:ABC-DEF..."></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('addChannelInstanceModal')">Cancel</button><button class="btn-o" onclick="doAddChannelInstance()">Add Channel</button></div>
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
async function showDash(){document.getElementById('loginView').style.display='none';document.getElementById('dashView').style.display='block';document.getElementById('sbUser').textContent=S.display_name||S.email;if(S.role==='admin'){document.getElementById('managementLabel').style.display='';document.getElementById('membersNav').style.display='';document.getElementById('usersNav').style.display='';document.getElementById('plansNav').style.display=''}await loadT();showPage('tenants')}
async function loadT(){const d=await api('GET','/api/portal/tenants');T=d.tenants||[]}

// Navigation
function showPage(p){D=null;document.querySelectorAll('.sbn .si').forEach(el=>el.classList.remove('active'));document.getElementById('headerActions').innerHTML='';history.pushState({page:p},'','/');
if(p==='tenants'){document.querySelector('.sbn .si:first-child').classList.add('active');document.getElementById('pageTitle').innerHTML='<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:24px;height:24px"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg> Tenants';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openCreateTenantModal()">+ Create Tenant</button>';renderList()}
else if(p==='channel-instances'){document.getElementById('channelInstancesNav').classList.add('active');document.getElementById('pageTitle').innerHTML='<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:22px;height:22px"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg> Multi Channels';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openAddChannelInstanceModal()">+ Add Channel</button>';renderChannelInstances()}
else if(p==='members'){document.getElementById('membersNav').classList.add('active');document.getElementById('pageTitle').textContent='Members';renderMembers()}
else if(p==='users'){document.getElementById('usersNav').classList.add('active');document.getElementById('pageTitle').textContent='Users';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openCreateUserModal()">+ Create User</button>';renderUsers()}
else if(p==='plans'){document.getElementById('plansNav').classList.add('active');document.getElementById('pageTitle').textContent='Service Plans';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openModal(\"createPlanModal\")">+ Create Plan</button>';renderPlans()}
else if(p==='workflows'){document.getElementById('workflowsNav').classList.add('active');document.getElementById('pageTitle').innerHTML='<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:22px;height:22px"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg> Workflows';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openModal(\"createWorkflowModal\")">+ Create Workflow</button>';renderWorkflows()}
else if(p==='scheduler'){document.getElementById('schedulerNav').classList.add('active');document.getElementById('pageTitle').innerHTML='<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:22px;height:22px"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> Scheduler';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openModal(\"createSchedulerModal\")">+ Create Job</button>';renderScheduler()}
else if(p==='agents'){document.getElementById('agentsNav').classList.add('active');document.getElementById('pageTitle').textContent='🤖 Agents';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openCreateAgentModal()">+ Tạo Agent</button>';renderAgentsList()}
else if(p==='knowledge'){document.getElementById('knowledgeNav').classList.add('active');document.getElementById('pageTitle').textContent='📚 Kho tri thức';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openKnowledgeUpload()">📎 Upload File</button>';renderKnowledge()}
else if(p==='tools'){document.getElementById('toolsNav').classList.add('active');document.getElementById('pageTitle').textContent='🛠️ Công cụ';renderTools()}
else if(p==='skills'){document.getElementById('skillsNav').classList.add('active');document.getElementById('pageTitle').textContent='🎯 Skills Market';renderSkills()}
else if(p==='gallery'){document.getElementById('galleryNav').classList.add('active');document.getElementById('pageTitle').textContent='🎨 Gallery';renderGallery()}
else if(p==='orchestration'){document.getElementById('orchestrationNav').classList.add('active');document.getElementById('pageTitle').textContent='🎯 Điều phối';renderOrchestration()}
else if(p==='orgmap'){document.getElementById('orgmapNav').classList.add('active');document.getElementById('pageTitle').textContent='🗺️ Org Map';renderOrgMap()}
else if(p==='kanban'){document.getElementById('kanbanNav').classList.add('active');document.getElementById('pageTitle').textContent='📋 Kanban Board';renderKanban()}
else if(p==='traces'){document.getElementById('tracesNav').classList.add('active');document.getElementById('pageTitle').textContent='📊 LLM Traces';renderTraces()}
else if(p==='cost'){document.getElementById('costNav').classList.add('active');document.getElementById('pageTitle').textContent='💰 Cost Tracking';renderCost()}
else if(p==='activity'){document.getElementById('activityNav').classList.add('active');document.getElementById('pageTitle').textContent='⚡ Activity Feed';document.getElementById('headerActions').innerHTML='<button class="btn-r" onclick="clearActivity()">🗑 Xoá Activity</button>';renderActivity()}
else if(p==='apikeys'){document.getElementById('apikeysNav').classList.add('active');document.getElementById('pageTitle').textContent='🔑 API Keys';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="createApiKey()">+ Tạo Key</button>';renderApiKeys()}
else if(p==='usage'){document.getElementById('usageNav').classList.add('active');document.getElementById('pageTitle').textContent='📊 Usage & Quotas';renderUsage()}
else if(p==='configfile'){document.getElementById('configfileNav').classList.add('active');document.getElementById('pageTitle').textContent='📄 Tệp cấu hình';renderConfigFile()}
}

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
  const memCount=(t.members||[]).length;
  const agentName=t.agent_name||t.name+' Agent';
  const msgUsed=t.messages_today||0;
  const msgMax=t.max_messages_per_day;
  const msgPct=msgMax>0&&msgMax<1e9?Math.min(100,Math.round(msgUsed/msgMax*100)):0;
  const msgColor=msgPct>=90?'var(--rt)':msgPct>=70?'#e67e22':'var(--gt)';
  let html=`<div class="cards">
    <div class="card"><div class="card-label">Status</div><div class="card-val"><span class="badge ${t.status}" style="font-size:.85rem;padding:4px 14px">${t.status==='running'?'Running':'Stopped'}</span></div></div>
    <div class="card"><div class="card-label">Agent</div><div class="card-val" style="font-size:.95rem;font-weight:600">${agentName}</div><div class="card-sub"><span class="badge ${t.status==='running'?'running':'stopped'}" style="font-size:.7rem">${t.status==='running'?'Online':'Offline'}</span> · ${t.provider||'groq'} / ${t.model||'-'}</div></div>
    <div class="card"><div class="card-label">Messages Today</div><div class="card-val" style="font-size:1.4rem;font-weight:700;color:${msgColor}">${msgUsed}<span style="font-size:.8rem;font-weight:400;color:var(--d)"> / ${fmt(msgMax)}</span></div><div style="margin-top:8px;background:var(--bg2);border-radius:4px;height:6px;overflow:hidden"><div style="width:${msgPct}%;height:100%;background:${msgColor};border-radius:4px;transition:width .3s"></div></div></div>
    <div class="card"><div class="card-label">Channels</div><div class="card-val">${chCount} / ${fmt(t.max_channels)}</div><div class="card-sub">${memCount} member${memCount!==1?'s':''}</div></div>
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
function openModal(id){document.getElementById(id).classList.add('show');if(id==='createWorkflowModal'&&document.getElementById('wfStepsContainer').children.length===0){addWfStep()}if(id==='createSchedulerModal'){const sel=document.getElementById('sjAgentId');sel.innerHTML='<option value="">Loading agents...</option>';loadAgentsList().then(agents=>{sel.innerHTML='<option value="">-- Select Agent --</option>';const tenantNames=(T||[]).map(t=>(t.agent_name||t.name||'').toLowerCase());const filtered=agents.filter(a=>{const n=(a.name||'').toLowerCase();return tenantNames.some(tn=>tn&&(n.includes(tn)||tn.includes(n)))});const list=filtered.length>0?filtered:agents;list.forEach(a=>{const o=document.createElement('option');o.value=a.name||a.id;o.textContent=(a.name||a.id)+(a.state?' ('+a.state+')':'');sel.appendChild(o)})})}}
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

// ─── Automation: Sidebar (section-label style) ──────────────────────────────

// ─── Automation: Workflows ───────────────────────────────────────────────────
const WF_TEMPLATES={'code-review':{name:'code-review-pipeline',desc:'Analyze code, review for issues, and produce a summary report',steps:[{name:'analyze',agent_name:'code-reviewer',prompt:'Analyze the following code for bugs, style issues, and security vulnerabilities:\n\n{{input}}',mode:'sequential',timeout_secs:180,error_mode:'fail',output_var:'analysis'},{name:'security-check',agent_name:'security-auditor',prompt:'Review this code analysis for security issues. Flag anything critical:\n\n{{analysis}}',mode:'sequential',timeout_secs:120,error_mode:'retry',max_retries:2,output_var:'security_review'},{name:'summary',agent_name:'writer',prompt:'Write a concise code review summary.\n\nCode Analysis:\n{{analysis}}\n\nSecurity Review:\n{{security_review}}',mode:'sequential',timeout_secs:60,error_mode:'fail'}]},'research-write':{name:'research-and-write',desc:'Research a topic, outline, write, and optionally fact-check',steps:[{name:'research',agent_name:'researcher',prompt:'Research the following topic thoroughly:\n\n{{input}}',mode:'sequential',timeout_secs:300,output_var:'research'},{name:'outline',agent_name:'planner',prompt:'Create a detailed article outline based on this research:\n\n{{research}}',mode:'sequential',timeout_secs:60,output_var:'outline'},{name:'write',agent_name:'writer',prompt:'Write a complete article.\n\nOutline:\n{{outline}}\n\nResearch:\n{{research}}',mode:'sequential',timeout_secs:300,output_var:'article'},{name:'fact-check',agent_name:'analyst',prompt:'Fact-check this article:\n\n{{article}}',mode:'conditional',condition:'claim',timeout_secs:120,error_mode:'skip'}]},'brainstorm':{name:'brainstorm',desc:'Parallel brainstorm with 3 agents, then synthesize',steps:[{name:'creative-ideas',agent_name:'writer',prompt:'Brainstorm 5 creative ideas for: {{input}}',mode:'fan_out',timeout_secs:60},{name:'technical-ideas',agent_name:'architect',prompt:'Brainstorm 5 technically feasible ideas for: {{input}}',mode:'fan_out',timeout_secs:60},{name:'business-ideas',agent_name:'analyst',prompt:'Brainstorm 5 ideas with strong business potential for: {{input}}',mode:'fan_out',timeout_secs:60},{name:'gather',agent_name:'planner',prompt:'unused',mode:'collect'},{name:'synthesize',agent_name:'orchestrator',prompt:'Synthesize brainstorm results into the top 5 actionable ideas:\n\n{{input}}',mode:'sequential',timeout_secs:120}]},'iterative':{name:'iterative-refinement',desc:'Refine a document until approved or max iterations reached',steps:[{name:'first-draft',agent_name:'writer',prompt:'Write a first draft about: {{input}}',mode:'sequential',timeout_secs:120,output_var:'draft'},{name:'review-and-refine',agent_name:'code-reviewer',prompt:'Review this draft. If it meets quality standards, respond with APPROVED. Otherwise, provide feedback and a revised version:\n\n{{input}}',mode:'loop',max_iterations:4,until:'APPROVED',timeout_secs:180,error_mode:'retry',max_retries:1}]}};

let wfStepCount=0;
let cachedAgents=null;
async function loadAgentsList(){if(cachedAgents)return cachedAgents;try{const d=await api('GET','/api/portal/system/agents');cachedAgents=Array.isArray(d)?d:[];return cachedAgents}catch(e){return []}}

function addWfStep(data){const c=document.getElementById('wfStepsContainer');const idx=wfStepCount++;const agName=data?data.agent_name||'':'';const prompt=data?data.prompt||data.prompt_template||'{{input}}':'{{input}}';const mode=data?data.mode||'sequential':'sequential';const errMode=data?data.error_mode||'fail':'fail';const sName=data?data.name||'step'+(idx+1):'step'+(idx+1);
const div=document.createElement('div');div.className='wf-step';div.dataset.idx=idx;
div.innerHTML=`<span class="step-num">STEP ${c.children.length+1}</span><button class="step-del" onclick="removeWfStep(this)">&times;</button><div style="margin-top:14px"><div class="config-row"><div class="fg" style="flex:1"><label>Step Name</label><input type="text" class="ws-name" value="${sName}"></div><div class="fg" style="flex:1"><label>Agent Name</label><div style="position:relative"><input type="text" class="ws-agent" value="${agName}" placeholder="e.g. writer" list="agentList${idx}"><datalist id="agentList${idx}"></datalist></div></div></div><div class="fg"><label>Prompt Template</label><textarea class="ws-prompt" rows="2" style="resize:vertical">${prompt}</textarea></div><div class="config-row"><div class="fg" style="flex:1"><label>Mode</label><select class="ws-mode"><option value="sequential"${mode==='sequential'?' selected':''}>Sequential</option><option value="fan_out"${mode==='fan_out'?' selected':''}>Fan Out (parallel)</option><option value="collect"${mode==='collect'?' selected':''}>Collect</option><option value="conditional"${mode==='conditional'?' selected':''}>Conditional</option><option value="loop"${mode==='loop'?' selected':''}>Loop</option></select></div><div class="fg" style="flex:1"><label>On Error</label><select class="ws-err"><option value="fail"${errMode==='fail'?' selected':''}>Fail</option><option value="skip"${errMode==='skip'?' selected':''}>Skip</option><option value="retry"${errMode==='retry'?' selected':''}>Retry</option></select></div></div></div>`;
c.appendChild(div);
loadAgentsList().then(agents=>{const dl=div.querySelector('datalist');agents.forEach(a=>{const o=document.createElement('option');o.value=a.name||a.id;dl.appendChild(o)})});
renumberSteps()}

function removeWfStep(btn){btn.closest('.wf-step').remove();renumberSteps()}
function renumberSteps(){document.querySelectorAll('#wfStepsContainer .wf-step').forEach((el,i)=>{el.querySelector('.step-num').textContent='STEP '+(i+1)})}

function fillWfTemplate(){const tpl=document.getElementById('wfTemplate').value;const c=document.getElementById('wfStepsContainer');c.innerHTML='';wfStepCount=0;if(tpl==='custom'){document.getElementById('wfName').value='';document.getElementById('wfDesc').value='';addWfStep();return}const t=WF_TEMPLATES[tpl];if(!t)return;document.getElementById('wfName').value=t.name;document.getElementById('wfDesc').value=t.desc;t.steps.forEach(s=>addWfStep(s))}

async function renderWorkflows(){
  const d=await api('GET','/api/portal/workflows');
  const wfs=Array.isArray(d)?d:(d.workflows||d.error?[]:[]);
  if(d.error){document.getElementById('mainContent').innerHTML=`<div class="sbox"><h3>Workflows</h3><div class="sbox-desc">Could not load workflows from OpenFang API.</div><div style="margin-top:8px;padding:12px;background:var(--rb);border-radius:8px;color:var(--rt);font-size:.85rem">${d.error}</div></div>`;return}
  if(wfs.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px 24px"><svg viewBox="0 0 24 24" fill="none" stroke="var(--m)" stroke-width="1.5" style="width:48px;height:48px;margin-bottom:16px"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg><h3 style="color:var(--d);font-weight:500">No workflows yet</h3><p style="color:var(--m);margin-top:8px;font-size:.85rem">Create a workflow to chain multiple agents together in a pipeline.</p><button class="btn-o" style="margin-top:16px" onclick="openModal('createWorkflowModal')">+ Create Workflow</button></div>`;return}
  const rows=wfs.map(w=>`<tr><td style="font-weight:500">${w.name||'-'}</td><td style="color:var(--d)">${w.description||'-'}</td><td><span class="badge plan">${w.steps||0} steps</span></td><td style="color:var(--d);font-size:.8rem">${fmtDate(w.created_at)}</td><td><button class="btn-g" onclick="openRunWorkflow('${w.id}')">Run</button> <button class="btn-g" onclick="viewWorkflowRuns('${w.id}')">Runs</button> <button class="btn-r" onclick="deleteWorkflow('${w.id}','${(w.name||'').replace(/'/g,"\\'")}')">Delete</button></td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total: <span class="sv">${wfs.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Description</th><th>Steps</th><th>Created</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}

async function doCreateWorkflow(){const name=document.getElementById('wfName').value.trim();const desc=document.getElementById('wfDesc').value.trim();if(!name){alert('Workflow name is required');return}const stepEls=document.querySelectorAll('#wfStepsContainer .wf-step');if(stepEls.length===0){alert('Add at least one step');return}const steps=[];for(const el of stepEls){const sn=el.querySelector('.ws-name').value.trim()||'step';const ag=el.querySelector('.ws-agent').value.trim();if(!ag){alert('Agent name is required for step "'+sn+'"');return}const pr=el.querySelector('.ws-prompt').value;const md=el.querySelector('.ws-mode').value;const em=el.querySelector('.ws-err').value;const step={name:sn,agent_name:ag,prompt:pr,mode:md,error_mode:em};steps.push(step)}const body={name,description:desc,steps};const d=await api('POST','/api/portal/workflows',body);if(d.workflow_id||d.id){closeModal('createWorkflowModal');renderWorkflows()}else{alert(d.error||'Failed to create workflow')}}

let runWfId=null;
function openRunWorkflow(id){runWfId=id;document.getElementById('wfRunInput').value='';document.getElementById('wfRunResult').innerHTML='';document.getElementById('wfRunBtn').disabled=false;openModal('runWorkflowModal')}
async function doRunWorkflow(){if(!runWfId)return;const input=document.getElementById('wfRunInput').value;document.getElementById('wfRunBtn').disabled=true;document.getElementById('wfRunResult').innerHTML='<div style="padding:12px;background:var(--bb);border-radius:8px;color:var(--bt);font-size:.85rem">⏳ Running workflow... This may take a while.</div>';try{const d=await api('POST','/api/portal/workflows/'+runWfId+'/run',{input});if(d.error){document.getElementById('wfRunResult').innerHTML=`<div style="padding:12px;background:var(--rb);border-radius:8px;color:var(--rt);font-size:.85rem">❌ ${d.error}</div>`}else{const output=d.output||JSON.stringify(d,null,2);document.getElementById('wfRunResult').innerHTML=`<div style="padding:12px;background:var(--gb);border-radius:8px;font-size:.85rem"><strong style="color:var(--gt)">✅ ${d.status||'completed'}</strong><pre style="margin-top:8px;white-space:pre-wrap;color:var(--t);font-size:.8rem">${output}</pre></div>`}}catch(e){document.getElementById('wfRunResult').innerHTML=`<div style="padding:12px;background:var(--rb);border-radius:8px;color:var(--rt);font-size:.85rem">❌ ${e}</div>`}finally{document.getElementById('wfRunBtn').disabled=false}}

async function viewWorkflowRuns(id){const d=await api('GET','/api/portal/workflows/'+id+'/runs');const runs=Array.isArray(d)?d:(d.runs||[]);if(runs.length===0){alert('No runs found for this workflow');return}let html='<div class="sbox"><h3>Workflow Runs</h3><table class="dt"><thead><tr><th>Run ID</th><th>State</th><th>Steps</th><th>Started</th><th>Completed</th></tr></thead><tbody>';runs.forEach(r=>{html+=`<tr><td style="font-family:monospace;font-size:.75rem">${(r.id||'').substring(0,8)}...</td><td><span class="badge ${r.state==='completed'?'running':r.state==='failed'?'stopped':'plan'}">${r.state||'-'}</span></td><td>${r.steps_completed||0}</td><td style="font-size:.8rem">${fmtDate(r.started_at)}</td><td style="font-size:.8rem">${fmtDate(r.completed_at)}</td></tr>`});html+='</tbody></table></div>';document.getElementById('mainContent').innerHTML=html}

async function deleteWorkflow(id,name){if(!confirm('Delete workflow "'+name+'"?'))return;const d=await api('DELETE','/api/portal/workflows/'+encodeURIComponent(id));renderWorkflows()}

// ─── Automation: Scheduler / Cron Jobs ────────────────────────────────────────

async function renderScheduler(){
  const d=await api('GET','/api/portal/scheduler');
  const jobs=Array.isArray(d)?d:(d.schedules||d.error?[]:[]);
  if(d.error){document.getElementById('mainContent').innerHTML=`<div class="sbox"><h3>Scheduler</h3><div class="sbox-desc">Could not load schedules from OpenFang API.</div><div style="margin-top:8px;padding:12px;background:var(--rb);border-radius:8px;color:var(--rt);font-size:.85rem">${d.error}</div></div>`;return}
  if(jobs.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px 24px"><svg viewBox="0 0 24 24" fill="none" stroke="var(--m)" stroke-width="1.5" style="width:48px;height:48px;margin-bottom:16px"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg><h3 style="color:var(--d);font-weight:500">No scheduled jobs yet</h3><p style="color:var(--m);margin-top:8px;font-size:.85rem">Create cron jobs to run agents on a schedule.</p><button class="btn-o" style="margin-top:16px" onclick="openModal('createSchedulerModal')">+ Create Job</button></div>`;return}
  const rows=jobs.map(j=>{const cron=j.cron||'';const agId=(j.agent_id||'').substring(0,8);const runs=j.run_count||0;return `<tr><td style="font-weight:500">${j.name||'-'}</td><td><span class="badge plan" style="font-family:monospace">${cron}</span></td><td style="font-family:monospace;font-size:.75rem">${agId}...</td><td>${runs}</td><td><span class="badge ${j.enabled!==false?'running':'stopped'}">${j.enabled!==false?'Enabled':'Disabled'}</span></td><td style="font-size:.8rem">${fmtDate(j.created_at)}</td><td><button class="btn-g" onclick="toggleSchedule('${j.id}',${!(j.enabled!==false)})">${j.enabled!==false?'Disable':'Enable'}</button> <button class="btn-r" onclick="deleteSchedule('${j.id}','${(j.name||'').replace(/'/g,"\\\\'")}')">Delete</button></td></tr>`}).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total: <span class="sv">${jobs.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Cron</th><th>Agent</th><th>Runs</th><th>Status</th><th>Created</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}

async function doCreateSchedulerJob(){const name=document.getElementById('sjName').value.trim();const cron=document.getElementById('sjCron').value.trim();const agentId=document.getElementById('sjAgentId').value;const message=document.getElementById('sjMessage').value.trim();if(!name){alert('Job name is required');return}if(!cron){alert('Cron expression is required');return}if(!agentId){alert('Please select a Target Agent');return}if(!message){alert('Message is required');return}const body={name,cron,agent_id:agentId,message,enabled:true};try{const d=await api('POST','/api/portal/scheduler',body);if(d.id||d.schedule_id||d.ok){closeModal('createSchedulerModal');document.getElementById('sjName').value='';document.getElementById('sjCron').value='';document.getElementById('sjMessage').value='';renderScheduler()}else{alert(d.error||JSON.stringify(d)||'Failed to create scheduled job')}}catch(e){alert('Error: '+e.message)}}

async function toggleSchedule(id,enabled){await api('PUT','/api/portal/scheduler/'+encodeURIComponent(id),{enabled});renderScheduler()}
async function deleteSchedule(id,name){if(!confirm('Delete scheduled job "'+name+'"?'))return;await api('DELETE','/api/portal/scheduler/'+encodeURIComponent(id));renderScheduler()}

// ─── Multi Channel Instances ─────────────────────────────────────────────────
const CH_ICONS={telegram:'✈️',zalo:'💬',discord:'🎮',slack:'💼',whatsapp:'📱',facebook:'📘',email:'📧',web:'🌐'};
const CH_COLORS={telegram:'#0088cc',zalo:'#0068ff',discord:'#5865F2',slack:'#4A154B',whatsapp:'#25D366',facebook:'#1877F2',email:'#EA4335',web:'#FF5C00'};

async function renderChannelInstances(){
  const d=await api('GET','/api/portal/channel-instances');
  const cis=d.channel_instances||[];
  const statusBadge=s=>s==='active'?'<span class="badge running">Active</span>':s==='error'?'<span class="badge stopped">Error</span>':s==='disabled'?'<span class="badge stopped">Disabled</span>':'<span class="badge plan">Pending</span>';
  if(cis.length===0){
    document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px 24px"><div style="font-size:3rem;margin-bottom:16px">📡</div><h3 style="color:var(--d);font-weight:500">No channel instances yet</h3><p style="color:var(--m);margin-top:8px;font-size:.85rem">Add independent channel connections (e.g., multiple Telegram bots) that route messages to your AI agents.<br>Unlike OpenFang channels, you can add <b>multiple instances</b> of the same type.</p><button class="btn-o" style="margin-top:16px" onclick="openAddChannelInstanceModal()">+ Add Channel Instance</button></div>`;
    return;
  }
  // Group by tenant
  const byTenant={};
  cis.forEach(ci=>{const tn=T.find(t=>t.id===ci.tenant_id);const tName=tn?tn.name:ci.tenant_id;if(!byTenant[tName])byTenant[tName]=[];byTenant[tName].push(ci)});
  let html=`<div class="sr"><span class="sl">Total Instances: <span class="sv">${cis.length}</span></span><span class="sl">Active: <span class="sv gn">${cis.filter(c=>c.status==='active').length}</span></span></div>`;
  for(const [tName,instances] of Object.entries(byTenant)){
    html+=`<div class="sbox" style="margin-bottom:16px"><h3 style="font-size:.95rem;font-weight:700;margin-bottom:12px">${tName}</h3>`;
    html+=`<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:12px">`;
    instances.forEach(ci=>{
      const icon=CH_ICONS[ci.channel_type]||'📡';
      const color=CH_COLORS[ci.channel_type]||'var(--o)';
      const hasToken=ci.config&&ci.config.bot_token;
      html+=`<div style="border:1px solid var(--b);border-radius:12px;padding:16px;background:var(--bg);position:relative;border-left:3px solid ${color}">`;
      html+=`<div style="display:flex;align-items:center;gap:10px;margin-bottom:10px"><span style="font-size:1.5rem">${icon}</span><div><div style="font-weight:600;font-size:.9rem">${ci.display_name}</div><div style="font-size:.75rem;color:var(--d);text-transform:capitalize">${ci.channel_type}</div></div><div style="margin-left:auto">${statusBadge(ci.status)}</div></div>`;
      html+=`<div style="display:flex;flex-direction:column;gap:4px;font-size:.78rem;color:var(--d);margin-bottom:10px">`;
      html+=`<div>Messages: <b style="color:var(--t)">${ci.message_count||0}</b></div>`;
      html+=`<div>Last: ${ci.last_message_at?fmtDate(ci.last_message_at):'Never'}</div>`;
      html+=`<div style="font-family:monospace;font-size:.7rem">Webhook: <code>${ci.webhook_path}</code></div>`;
      html+=`</div>`;
      html+=`<div style="display:flex;gap:6px;flex-wrap:wrap">`;
      if(ci.channel_type==='telegram'){
        html+=`<button class="btn-g" style="font-size:.75rem;padding:4px 10px" onclick="testChannelInstance('${ci.id}')">🔍 Test</button>`;
        html+=`<button class="btn-g" style="font-size:.75rem;padding:4px 10px" onclick="setChannelWebhook('${ci.id}')">🔗 Set Webhook</button>`;
      }
      html+=`<button class="btn-g" style="font-size:.75rem;padding:4px 10px" onclick="configChannelInstance('${ci.id}')">⚙️ Config</button>`;
      html+=`<button class="btn-r" style="font-size:.75rem" onclick="deleteChannelInstance('${ci.id}','${ci.display_name.replace(/'/g,"\\'")}')">Delete</button>`;
      html+=`</div></div>`;
    });
    html+=`</div></div>`;
  }
  document.getElementById('mainContent').innerHTML=html;
}

function openAddChannelInstanceModal(){
  const sel=document.getElementById('ciTenant');
  sel.innerHTML=T.map(t=>`<option value="${t.id}">${t.name}</option>`).join('');
  document.getElementById('ciName').value='';
  document.getElementById('ciToken').value='';
  openModal('addChannelInstanceModal');
}

async function doAddChannelInstance(){
  const tenantId=document.getElementById('ciTenant').value;
  const type=document.getElementById('ciType').value;
  const name=document.getElementById('ciName').value.trim();
  const token=document.getElementById('ciToken').value.trim();
  if(!name){alert('Display name is required');return}
  const config={};
  if(type==='telegram'&&token)config.bot_token=token;
  const d=await api('POST','/api/portal/channel-instances',{tenant_id:tenantId,channel_type:type,display_name:name,config});
  if(d.ok){
    closeModal('addChannelInstanceModal');
    renderChannelInstances();
    if(type==='telegram'&&token){
      // Auto-test after creation
      setTimeout(()=>testChannelInstance(d.id),500);
    }
  } else alert(d.error||'Failed');
}

async function testChannelInstance(id){
  const d=await api('POST','/api/portal/channel-instances/'+id+'/test');
  if(d.ok){
    const info=d.bot_info||{};
    alert('✅ Bot verified!\n\nName: '+( info.first_name||'-')+'\nUsername: @'+(info.username||'-'));
    renderChannelInstances();
  } else alert('❌ Test failed: '+(d.error||'Unknown error'));
}

async function setChannelWebhook(id){
  const baseUrl=prompt('Enter your Portal public URL (e.g. https://portal.openfang.com.vn):',location.origin);
  if(!baseUrl)return;
  const d=await api('POST','/api/portal/channel-instances/'+id+'/webhook',{base_url:baseUrl});
  if(d.ok){
    alert('✅ Webhook set!\n\nURL: '+d.webhook_url);
    renderChannelInstances();
  } else alert('❌ Failed: '+(d.error||'Unknown'));
}

async function configChannelInstance(id){
  const d=await api('GET','/api/portal/channel-instances/'+id);
  if(d.error){alert(d.error);return}
  const cfg=d.config||{};
  const token=prompt('Bot Token:',cfg.bot_token||'');
  if(token===null)return;
  const upd=await api('PUT','/api/portal/channel-instances/'+id,{config:{bot_token:token}});
  if(upd.ok){renderChannelInstances()}else{alert(upd.error||'Failed')}
}

async function deleteChannelInstance(id,name){
  if(!confirm('Delete channel "'+name+'"? The webhook will be removed from Telegram.'))return;
  const d=await api('DELETE','/api/portal/channel-instances/'+id);
  if(d.ok)renderChannelInstances();else alert(d.error||'Failed');
}

// ─── Multi-Agent Management ──────────────────────────────────────────────────
let _currentAgentsTenantId = '';
async function renderAgentsList(){
  const data = await api('GET','/api/portal/tenants');
  const tenants = data.tenants || [];
  if(tenants.length === 0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><h3 style="color:var(--d)">Chưa có tenant</h3><p style="color:var(--m);font-size:.85rem">Tạo tenant trước khi thêm agents.</p></div>`;return}

  // Tenant selector
  if(!_currentAgentsTenantId) _currentAgentsTenantId = tenants[0].id;
  const select = tenants.map(t=>`<option value="${t.id}" ${t.id===_currentAgentsTenantId?'selected':''}>${t.name}</option>`).join('');

  // Load agents for selected tenant
  const agentData = await api('GET','/api/portal/agents?tenant_id='+_currentAgentsTenantId);
  const agents = agentData.agents || [];

  // Load channels for assignment display
  const chData = await api('GET','/api/portal/channel-instances?tenant_id='+_currentAgentsTenantId);
  const channels = (chData.instances || []).filter(c=>c.tenant_id===_currentAgentsTenantId);

  const cards = agents.map(a=>{
    const linkedChs = channels.filter(c=>c.agent_id===a.id);
    const chBadges = linkedChs.map(c=>`<span class="badge running" style="font-size:.65rem">${c.display_name}</span>`).join(' ');
    return `<div style="border:1px solid var(--b);border-radius:12px;padding:16px;background:var(--bg)">
      <div style="display:flex;align-items:center;gap:10px;margin-bottom:8px">
        <span style="font-size:2rem">${a.icon||'🤖'}</span>
        <div>
          <div style="font-weight:600">${a.name}</div>
          <div style="font-size:.7rem;color:var(--d)">${a.role||'assistant'} • ${a.model||'-'}</div>
        </div>
        <span class="badge ${a.enabled?'running':'stopped'}" style="margin-left:auto">${a.enabled?'Active':'Off'}</span>
      </div>
      <div style="font-size:.75rem;color:var(--d);margin-bottom:6px">Skills: ${(a.skills||[]).join(', ')||'none'}</div>
      <div style="font-size:.75rem;color:var(--d);margin-bottom:8px">Channels: ${chBadges||'<span style=\"color:var(--m)\">unlinked</span>'}</div>
      <div style="display:flex;gap:6px">
        <button class="btn-g" style="font-size:.75rem" onclick="editAgent('${a.id}')">✏️ Edit</button>
        <button class="btn-r" style="font-size:.75rem" onclick="deleteAgent('${a.id}')">🗑</button>
      </div>
    </div>`;
  }).join('');

  const noAgentMsg = agents.length===0?`<div class="sbox" style="text-align:center;padding:32px"><div style="font-size:2rem;margin-bottom:8px">🤖</div><p style="color:var(--d);font-size:.85rem">Chưa có agent. Bấm "+ Tạo Agent" để thêm.</p></div>`:'';

  document.getElementById('mainContent').innerHTML=`
    <div style="margin-bottom:16px;display:flex;align-items:center;gap:12px">
      <label style="font-weight:600;font-size:.85rem">Tenant:</label>
      <select onchange="_currentAgentsTenantId=this.value;renderAgentsList()" style="padding:6px 12px;border:1px solid var(--b);border-radius:8px;background:var(--bg);color:var(--t);font-size:.85rem">${select}</select>
      <span class="sl">Agents: <span class="sv">${agents.length}</span></span>
    </div>
    ${noAgentMsg}
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:12px">${cards}</div>`;
}

function openCreateAgentModal(){
  const name=prompt('Tên Agent:');if(!name)return;
  const role=prompt('Vai trò (assistant, sales, support, analyst):','assistant')||'assistant';
  api('POST','/api/portal/agents',{tenant_id:_currentAgentsTenantId,name,role}).then(d=>{
    if(d.ok)renderAgentsList();else alert(d.error||'Failed');
  });
}

async function editAgent(id){
  const agentData = await api('GET','/api/portal/agents?tenant_id='+_currentAgentsTenantId);
  const agent = (agentData.agents||[]).find(a=>a.id===id);
  if(!agent)return alert('Agent not found');
  const name=prompt('Tên Agent:',agent.name);if(!name)return;
  const role=prompt('Vai trò:',agent.role)||agent.role;
  const prompt_text=prompt('System Prompt:',agent.system_prompt)||agent.system_prompt;
  await api('PUT','/api/portal/agents/'+id,{name,role,system_prompt:prompt_text});
  renderAgentsList();
}

async function deleteAgent(id){
  if(!confirm('Xoá agent này? Channels liên kết sẽ bị unlink.'))return;
  await api('DELETE','/api/portal/agents/'+id);
  renderAgentsList();
}

// ─── Knowledge Base (RAG) ────────────────────────────────────────────────────
async function renderKnowledge(){
  const d=await api('GET','/api/portal/knowledge');
  const docs=d.documents||[];
  if(docs.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px 24px"><div style="font-size:3rem;margin-bottom:16px">📚</div><h3 style="color:var(--d);font-weight:500">Chưa có tài liệu</h3><p style="color:var(--m);margin-top:8px;font-size:.85rem">Upload PDF hoặc paste text để AI trả lời chính xác hơn (RAG).</p><button class="btn-o" style="margin-top:16px" onclick="openKnowledgeUpload()">📎 Upload File</button></div>`;return}
  const rows=docs.map(doc=>`<tr><td style="font-weight:500">${doc.name||doc.filename||'-'}</td><td><span class="badge plan">${doc.chunks||0} chunks</span></td><td style="font-size:.8rem;color:var(--d)">${doc.size||'-'}</td><td style="font-size:.8rem;color:var(--d)">${fmtDate(doc.created_at)}</td><td><button class="btn-r" onclick="deleteKnowledge('${doc.id}')">Delete</button></td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Tài liệu: <span class="sv">${d.total_docs||docs.length}</span></span><span class="sl">Đoạn văn: <span class="sv">${d.total_chunks||0}</span></span></div><table class="dt"><thead><tr><th>Tên</th><th>Chunks</th><th>Size</th><th>Ngày tạo</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
function openKnowledgeUpload(){const text=prompt('Nhập nội dung text để thêm vào Knowledge Base:');if(!text)return;api('POST','/api/portal/knowledge',{content:text,type:'text'}).then(d=>{if(d.ok||d.id)renderKnowledge();else alert(d.error||'Failed')})}
async function deleteKnowledge(id){if(!confirm('Xoá tài liệu này?'))return;await api('DELETE','/api/portal/knowledge/'+id);renderKnowledge()}

// ─── Tools ───────────────────────────────────────────────────────────────────
async function renderTools(){
  const d=await api('GET','/api/portal/tools');
  const tools=d.tools||[];
  if(tools.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><h3 style="color:var(--d)">Không có tools</h3><p style="color:var(--m);font-size:.85rem">Chưa có công cụ nào từ OpenFang API.</p></div>`;return}
  const cards=tools.map(t=>`<div style="border:1px solid var(--b);border-radius:12px;padding:16px;background:var(--bg)"><div style="display:flex;align-items:center;gap:10px;margin-bottom:8px"><span style="font-size:1.5rem">${t.icon||'🔧'}</span><div><div style="font-weight:600">${t.name}</div><div style="font-size:.75rem;color:var(--d)">${t.desc||''}</div></div></div><div style="display:flex;justify-content:space-between;align-items:center"><span class="badge ${t.enabled?'running':'stopped'}">${t.enabled?'Enabled':'Disabled'}</span><button class="btn-g" style="font-size:.75rem" onclick="toggleTool('${t.name}',${!t.enabled})">${t.enabled?'Disable':'Enable'}</button></div></div>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total: <span class="sv">${tools.length}</span></span><span class="sl">Active: <span class="sv gn">${tools.filter(t=>t.enabled).length}</span></span></div><div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:12px">${cards}</div>`;
}
async function toggleTool(name,enable){await api('POST','/api/portal/tools/'+name+'/toggle',{enabled:enable});renderTools()}

// ─── Skills Market ───────────────────────────────────────────────────────────
async function renderSkills(){
  const d=await api('GET','/api/portal/system/skills');
  const skills=d.skills||d||[];
  if(!Array.isArray(skills)||skills.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><div style="font-size:3rem;margin-bottom:16px">🎯</div><h3 style="color:var(--d)">Skills Market</h3><p style="color:var(--m);font-size:.85rem">Chưa có skills từ OpenFang API.</p></div>`;return}
  const CATS={coding:'💻',data:'📊',security:'🔒',business:'💼',writing:'✍️',research:'🔬'};
  const cards=skills.map(s=>`<div style="border:1px solid var(--b);border-radius:12px;padding:16px;background:var(--bg)"><div style="display:flex;align-items:center;gap:10px;margin-bottom:8px"><span style="font-size:1.5rem">${s.icon||CATS[s.category]||'🎯'}</span><div><div style="font-weight:600">${s.name}</div><div style="font-size:.7rem;color:var(--d)">${s.category||''} • v${s.version||'1.0'}</div></div></div><p style="font-size:.8rem;color:var(--d);margin-bottom:10px">${s.description||''}</p><span class="badge ${s.installed?'running':'plan'}">${s.installed?'Installed':'Available'}</span></div>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total: <span class="sv">${skills.length}</span></span></div><div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:12px">${cards}</div>`;
}

// ─── Gallery (Agent Templates) ────────────────────────────────────────────────
async function renderGallery(){
  const d=await api('GET','/api/portal/gallery');
  const items=d.templates||d.gallery||d||[];
  if(!Array.isArray(items)||items.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><div style="font-size:3rem;margin-bottom:16px">🎨</div><h3 style="color:var(--d)">Agent Gallery</h3><p style="color:var(--m);font-size:.85rem">Mẫu agent templates từ OpenFang.</p></div>`;return}
  const cards=items.map(t=>`<div style="border:1px solid var(--b);border-radius:12px;padding:16px;background:var(--bg)"><div style="font-weight:600;margin-bottom:4px">${t.icon||'🤖'} ${t.name||t.title||'-'}</div><div style="font-size:.75rem;color:var(--d)">${t.category||t.department||''}</div><p style="font-size:.8rem;color:var(--d);margin:8px 0">${t.description||''}</p><button class="btn-g" style="font-size:.75rem">Clone</button></div>`).join('');
  document.getElementById('mainContent').innerHTML=`<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:12px">${cards}</div>`;
}

// ─── Orchestration ───────────────────────────────────────────────────────────
async function renderOrchestration(){
  const d=await api('GET','/api/portal/orchestration');
  const links=['delegate','handoff','broadcast','escalate'];
  const delegations=d.delegations||[];
  document.getElementById('mainContent').innerHTML=`<div style="display:grid;grid-template-columns:1fr 1fr;gap:16px"><div class="sbox"><h3 style="margin-bottom:12px">📋 Ủy quyền (${delegations.length})</h3>${delegations.length===0?'<p style="color:var(--d);font-size:.85rem">Chưa có delegation.</p>':delegations.map(d=>`<div style="padding:8px;border:1px solid var(--b);border-radius:8px;margin-bottom:6px"><b>${d.name||d.from||'-'}</b> → ${d.to||'-'}</div>`).join('')}</div><div class="sbox"><h3 style="margin-bottom:12px">🔗 Liên kết quyền</h3>${links.map(l=>`<div style="display:flex;justify-content:space-between;align-items:center;padding:8px;border:1px solid var(--b);border-radius:8px;margin-bottom:6px"><span style="font-weight:600">${l}</span><span class="badge running">Enabled</span></div>`).join('')}</div></div>`;
}

// ─── Org Map ─────────────────────────────────────────────────────────────────
async function renderOrgMap(){
  const d=await api('GET','/api/portal/orgmap');
  const nodes=d.nodes||[];
  if(nodes.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><div style="font-size:3rem;margin-bottom:16px">🗺️</div><h3 style="color:var(--d)">Org Map</h3><p style="color:var(--m);font-size:.85rem">Sơ đồ tổ chức Agent. Tạo tenants & agents để xem hierarchy.</p></div>`;return}
  const tenants=nodes.filter(n=>n.type==='tenant');
  let html='';
  tenants.forEach(t=>{
    const agents=nodes.filter(n=>n.type==='agent'&&n.parent===t.id);
    const agentCards=agents.map(a=>`<div style="border:1px solid var(--b);border-radius:10px;padding:12px;background:var(--bg);text-align:center;min-width:120px"><div style="font-size:1.5rem">${a.icon||'🤖'}</div><div style="font-weight:600;font-size:.8rem;margin-top:2px">${a.name}</div><div style="font-size:.65rem;color:var(--d)">${a.role||'agent'}</div></div>`).join('');
    html+=`<div class="sbox" style="margin-bottom:16px"><div style="display:flex;align-items:center;gap:10px;margin-bottom:12px"><span style="font-size:1.5rem">${t.icon||'🏢'}</span><div><div style="font-weight:700">${t.name}</div><div style="font-size:.7rem;color:var(--d)">Status: ${t.role}</div></div></div><div style="border-left:2px solid var(--o);margin-left:20px;padding-left:16px"><div style="display:flex;flex-wrap:wrap;gap:10px">${agentCards}</div></div></div>`;
  });
  document.getElementById('mainContent').innerHTML=html;
}

// ─── Kanban ──────────────────────────────────────────────────────────────────
async function renderKanban(){
  const d=await api('GET','/api/portal/kanban');
  const cols={'inbox':[],'in_progress':[],'review':[],'done':[]};
  const tasks=d.tasks||d.items||[];
  tasks.forEach(t=>{const col=t.status||'inbox';if(cols[col])cols[col].push(t);else cols['inbox'].push(t)});
  const colNames={'inbox':'📥 Inbox','in_progress':'🔄 Đang làm','review':'👀 Review','done':'✅ Hoàn thành'};
  let html='<div style="display:grid;grid-template-columns:repeat(4,1fr);gap:12px;min-height:400px">';
  for(const [key,items] of Object.entries(cols)){
    html+=`<div style="background:var(--ol);border-radius:12px;padding:12px"><h4 style="font-size:.85rem;margin-bottom:8px">${colNames[key]||key} (${items.length})</h4>`;
    items.forEach(t=>{html+=`<div style="background:var(--bg);border:1px solid var(--b);border-radius:8px;padding:10px;margin-bottom:8px"><div style="font-weight:500;font-size:.85rem">${t.title||t.name||'-'}</div><div style="font-size:.7rem;color:var(--d);margin-top:4px">${t.agent||''}</div></div>`});
    html+='</div>';
  }
  html+='</div>';
  document.getElementById('mainContent').innerHTML=html;
}

// ─── LLM Traces ──────────────────────────────────────────────────────────────
async function renderTraces(){
  const d=await api('GET','/api/portal/traces');
  const traces=d.traces||d||[];
  if(!Array.isArray(traces)||traces.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><div style="font-size:3rem;margin-bottom:16px">📊</div><h3 style="color:var(--d)">LLM Traces</h3><p style="color:var(--m);font-size:.85rem">Giám sát mọi cuộc gọi LLM: token, latency, cost.</p></div>`;return}
  const rows=traces.map(t=>`<tr><td style="font-family:monospace;font-size:.75rem">${(t.id||'').substring(0,8)}...</td><td>${t.model||'-'}</td><td>${t.prompt_tokens||0}</td><td>${t.completion_tokens||0}</td><td>${t.latency_ms||0}ms</td><td>$${(t.cost||0).toFixed(4)}</td><td style="font-size:.8rem">${fmtDate(t.created_at)}</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<table class="dt"><thead><tr><th>ID</th><th>Model</th><th>Prompt</th><th>Completion</th><th>Latency</th><th>Cost</th><th>Time</th></tr></thead><tbody>${rows}</tbody></table>`;
}

// ─── Cost Tracking ───────────────────────────────────────────────────────────
async function renderCost(){
  const d=await api('GET','/api/portal/cost');
  const models=d.models||d.breakdown||[];
  const total=d.total_cost||0;
  if(!Array.isArray(models)||models.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><div style="font-size:3rem;margin-bottom:16px">💰</div><h3 style="color:var(--d)">Cost Tracking</h3><p style="color:var(--m);font-size:.85rem">Thống kê chi phí LLM theo model. Bắt đầu chat để tạo traces.</p></div>`;return}
  const rows=models.map(m=>`<tr><td style="font-weight:500">${m.model||m.name||'-'}</td><td>${m.requests||0}</td><td>${m.tokens||0}</td><td>$${(m.cost||0).toFixed(4)}</td><td>${(m.percentage||0).toFixed(1)}%</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Cost: <span class="sv">$${total.toFixed(4)}</span></span></div><table class="dt"><thead><tr><th>Model</th><th>Requests</th><th>Tokens</th><th>Cost</th><th>%</th></tr></thead><tbody>${rows}</tbody></table>`;
}

// ─── Activity Feed ───────────────────────────────────────────────────────────
async function renderActivity(){
  const d=await api('GET','/api/portal/activity');
  const events=d.events||d||[];
  if(!Array.isArray(events)||events.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><div style="font-size:3rem;margin-bottom:16px">⚡</div><h3 style="color:var(--d)">Activity Feed</h3><p style="color:var(--m);font-size:.85rem">Nhật ký sự kiện hệ thống. Tự động refresh.</p></div>`;return}
  const items=events.map(e=>`<div style="border-left:3px solid var(--o);padding:8px 12px;margin-bottom:8px;background:var(--ol);border-radius:0 8px 8px 0"><div style="font-weight:500;font-size:.85rem">${e.type||e.event||'-'}</div><div style="font-size:.75rem;color:var(--d)">${e.message||e.detail||''}</div><div style="font-size:.7rem;color:var(--m);margin-top:2px">${fmtDate(e.timestamp||e.created_at)}</div></div>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Events: <span class="sv">${events.length}</span></span></div>${items}`;
}
async function clearActivity(){if(!confirm('Xoá tất cả activity?'))return;await api('DELETE','/api/portal/activity');renderActivity()}

// ─── API Keys ────────────────────────────────────────────────────────────────
async function renderApiKeys(){
  const d=await api('GET','/api/portal/apikeys');
  const keys=d.keys||d.api_keys||d||[];
  if(!Array.isArray(keys)||keys.length===0){document.getElementById('mainContent').innerHTML=`<div class="sbox" style="text-align:center;padding:48px"><div style="font-size:3rem;margin-bottom:16px">🔑</div><h3 style="color:var(--d)">API Keys</h3><p style="color:var(--m);font-size:.85rem">Tạo API key để truy cập từ ứng dụng bên ngoài.</p><button class="btn-o" style="margin-top:16px" onclick="createApiKey()">+ Tạo Key</button></div>`;return}
  const rows=keys.map(k=>`<tr><td style="font-weight:500">${k.name||'-'}</td><td style="font-family:monospace;font-size:.75rem">${k.key?k.key.substring(0,12)+'...':k.prefix||'-'}</td><td style="font-size:.8rem">${fmtDate(k.created_at)}</td><td><button class="btn-r" onclick="deleteApiKey('${k.id}')">Delete</button></td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<table class="dt"><thead><tr><th>Name</th><th>Key</th><th>Created</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
async function createApiKey(){const name=prompt('Tên cho API key:');if(!name)return;const d=await api('POST','/api/portal/apikeys',{name});if(d.key||d.id){alert('✅ Key tạo thành công!'+(d.key?'\n\nKey: '+d.key:''));renderApiKeys()}else alert(d.error||'Failed')}
async function deleteApiKey(id){if(!confirm('Xoá API key?'))return;await api('DELETE','/api/portal/apikeys/'+id);renderApiKeys()}

// ─── Usage & Quotas ──────────────────────────────────────────────────────────
async function renderUsage(){
  const d=await api('GET','/api/portal/usage');
  const metrics=[
    {label:'Agents',used:d.agents_used||0,max:d.agents_max||10,icon:'🤖'},
    {label:'Tokens/tháng',used:d.tokens_used||0,max:d.tokens_max||100000,icon:'📊'},
    {label:'Requests/tháng',used:d.requests_used||0,max:d.requests_max||10000,icon:'📨'},
    {label:'API Keys',used:d.apikeys_used||0,max:d.apikeys_max||5,icon:'🔑'}
  ];
  const bars=metrics.map(m=>{const pct=m.max>0?Math.min(100,m.used/m.max*100):0;const color=pct>80?'var(--rt)':pct>50?'#f59e0b':'var(--gt)';return `<div class="sbox" style="margin-bottom:12px"><div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px"><span style="font-weight:600">${m.icon} ${m.label}</span><span style="font-size:.85rem;color:var(--d)">${m.used.toLocaleString()} / ${m.max.toLocaleString()}</span></div><div style="background:var(--ol);border-radius:8px;height:8px;overflow:hidden"><div style="height:100%;width:${pct}%;background:${color};border-radius:8px;transition:width .3s"></div></div></div>`}).join('');
  document.getElementById('mainContent').innerHTML=bars;
}

// ─── Config File ─────────────────────────────────────────────────────────────
async function renderConfigFile(){
  const d=await api('GET','/api/portal/configfile');
  const content=d.content||d.config||JSON.stringify(d,null,2);
  document.getElementById('mainContent').innerHTML=`<div class="sbox"><h3 style="margin-bottom:12px">📄 config.toml</h3><textarea id="configContent" style="width:100%;height:400px;font-family:monospace;font-size:.8rem;padding:12px;border:1px solid var(--b);border-radius:8px;background:var(--ol);color:var(--t);resize:vertical">${typeof content==='string'?content:JSON.stringify(content,null,2)}</textarea><div style="margin-top:12px;text-align:right"><button class="btn-o" onclick="saveConfigFile()">💾 Save Config</button></div></div>`;
}
async function saveConfigFile(){const content=document.getElementById('configContent').value;const d=await api('POST','/api/portal/configfile',{content});if(d.ok)alert('✅ Config saved!');else alert(d.error||'Failed')}

// Init + Permalink
window.addEventListener('popstate',function(e){if(e.state&&e.state.page==='detail'&&e.state.id){openDetail(e.state.id)}else{showPage('tenants')}});
// Terminal typing animation
const TERM_LINES=[
  {type:'cmd',text:'$ openfang serve'},
  {type:'info',text:'  compiling openfang v0.9.2...'},
  {type:'ok',text:'  ✓ config loaded from openfang.toml'},
  {type:'ok',text:'  ✓ database connected (pool: 8)'},
  {type:'ok',text:'  ✓ hands 7 active — browser, researcher, collector...'},
  {type:'ok',text:'  ✓ skills 12 loaded'},
  {type:'warn',text:'  ⚡ booted in 182ms'},
  {type:'ok',text:'  ✓ gateway ready on :3000'},
  {type:'info',text:'  waiting for connections...'}
];
let termRunning=false;
function typeTerminal(){
  if(termRunning)return;termRunning=true;
  const el=document.getElementById('terminalBody');
  if(!el){termRunning=false;return}
  el.innerHTML='';
  let lineIdx=0;
  function typeLine(){
    if(lineIdx>=TERM_LINES.length){termRunning=false;setTimeout(()=>{if(document.getElementById('loginView').style.display!=='none')typeTerminal()},3000);return}
    const ln=TERM_LINES[lineIdx];
    const div=document.createElement('div');
    div.className='line visible';
    el.appendChild(div);
    let charIdx=0;
    const speed=ln.type==='cmd'?60:20;
    function typeChar(){
      if(charIdx<=ln.text.length){
        const txt=ln.text.substring(0,charIdx);
        if(ln.type==='cmd'){const parts=txt.split(' ');div.innerHTML='<span class="prompt">'+parts[0]+'</span> <span class="cmd">'+(parts.slice(1).join(' '))+'</span><span class="cursor"></span>'}
        else if(ln.type==='ok')div.innerHTML='<span class="ok">'+txt+'</span>';
        else if(ln.type==='warn')div.innerHTML='<span class="warn">'+txt+'</span>';
        else div.innerHTML='<span class="info">'+txt+'</span>';
        charIdx++;setTimeout(typeChar,speed+(Math.random()*20-10))
      }else{
        div.innerHTML=div.innerHTML.replace('<span class="cursor"></span>','');
        lineIdx++;setTimeout(typeLine,ln.type==='cmd'?500:150)
      }
    }
    typeChar();
  }
  typeLine();
}
(function(){const s=localStorage.getItem('ps');if(s){try{S=JSON.parse(s);
  const m=location.pathname.match(/\/([a-f0-9-]{36})/i);
  if(m){showDash().then(()=>openDetail(m[1]))}else{showDash()}
}catch(e){localStorage.removeItem('ps')}}if(document.getElementById('loginView').style.display!=='none')typeTerminal()})();
</script>
</body></html>"##;

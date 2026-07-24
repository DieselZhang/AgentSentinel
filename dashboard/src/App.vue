<template>
  <div id="app-container">
    <nav class="top-nav">
      <router-link to="/" class="logo">AgentSentinel</router-link>
      <div class="nav-links">
        <router-link to="/">Runs</router-link>
        <router-link to="/upload">Upload</router-link>
      </div>
    </nav>
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f1117; color: #e1e4e8; }
.top-nav { display: flex; align-items: center; justify-content: space-between; padding: 12px 24px; background: #161b22; border-bottom: 1px solid #30363d; }
.top-nav .logo { font-size: 18px; font-weight: 700; color: #58a6ff; text-decoration: none; }
.nav-links { display: flex; gap: 16px; }
.nav-links a { color: #8b949e; text-decoration: none; font-size: 14px; }
.nav-links a:hover, .nav-links a.router-link-exact-active { color: #e1e4e8; }
.main-content { max-width: 1200px; margin: 0 auto; padding: 24px; }
</style>

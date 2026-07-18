<script lang="ts">
    import { page } from '$app/stores';
</script>

<nav class="flex w-16 flex-col items-center gap-4 py-6 shrink-0 h-screen bg-surface overflow-y-auto">
    <a
        href="/"
        class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary text-on-primary transition-all hover:brightness-110 active:scale-95"
        onclick={() => window.location.reload()}
        title="Reload"
    >
        <span class="material-symbols-outlined text-[18px]">refresh</span>
    </a>

    {#each [
      { href: '/', icon: 'devices', label: 'Devices' },
      { href: '/apps', icon: 'apps', label: 'Apps' },
      { href: '/files', icon: 'folder_open', label: 'Files' },
      { href: '/process-manager', icon: 'memory', label: 'Process' },
      { href: '/shell', icon: 'terminal', label: 'Shell' },
      { href: '/flasher', icon: 'bolt', label: 'Flasher' },
      { href: '/wireless', icon: 'settings_input_antenna', label: 'Wireless' },
      { href: '/audit', icon: 'history', label: 'Audit' },
    ] as const as item}
      <a
        href={item.href}
        class="group flex flex-col items-center gap-1 no-underline"
      >
        <div
          class="flex h-8 w-12 items-center justify-center rounded-lg {$page.url.pathname === item.href ? 'bg-primary/15 text-primary' : 'text-on-surface-variant/60 transition-colors group-hover:text-on-surface'}"
        >
          <span class="material-symbols-outlined text-[18px]" style="font-variation-settings: {$page.url.pathname === item.href ? "'FILL' 1" : "'FILL' 0"};">{item.icon}</span>
        </div>
      </a>
    {/each}
</nav>

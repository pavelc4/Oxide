<script lang="ts">
    import { page } from '$app/stores';
</script>

<nav class="flex w-16 flex-col items-center justify-between py-6 shrink-0 h-screen bg-surface">
    <!-- Top Links -->
    <div class="flex flex-col items-center gap-4 overflow-y-auto min-h-0">
        <a
            href="/"
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary text-on-primary transition-all hover:brightness-110 active:scale-95 shadow-sm"
            title="Oxide"
        >
            <span class="material-symbols-outlined text-[20px]">science</span>
        </a>

        {#each [
          { href: '/', icon: 'devices', label: 'Devices' },
          { href: '/apps', icon: 'apps', label: 'Apps' },
          { href: '/files', icon: 'folder_open', label: 'Files' },
          { href: '/process-manager', icon: 'memory', label: 'Process' },
          { href: '/shell', icon: 'terminal', label: 'Shell' },
          { href: '/flasher', icon: 'bolt', label: 'Flasher' },
          { href: '/logcat', icon: 'subtitles', label: 'Logcat' },
          { href: '/audit', icon: 'history', label: 'Audit' },
        ] as const as item}
          <a
            href={item.href}
            class="group flex flex-col items-center gap-1 no-underline"
            title={item.label}
          >
            <div
              class="flex h-8 w-12 items-center justify-center rounded-lg {$page.url.pathname === item.href ? 'bg-primary/15 text-primary' : 'text-on-surface-variant/60 transition-colors group-hover:text-on-surface'}"
            >
              <span class="material-symbols-outlined text-[18px]" style="font-variation-settings: {$page.url.pathname === item.href ? "'FILL' 1" : "'FILL' 0"};">{item.icon}</span>
            </div>
          </a>
        {/each}
    </div>

    <!-- Bottom Settings Link -->
    <a
        href="/settings"
        class="group flex flex-col items-center gap-1 no-underline mt-auto pt-2"
        title="Settings"
    >
        <div
            class="flex h-10 w-10 items-center justify-center rounded-xl {$page.url.pathname === '/settings' ? 'bg-primary/15 text-primary' : 'text-on-surface-variant/70 hover:bg-surface-container-high hover:text-on-surface'} transition-all"
        >
            <span class="material-symbols-outlined text-[20px]" style="font-variation-settings: {$page.url.pathname === '/settings' ? "'FILL' 1" : "'FILL' 0"};">settings</span>
        </div>
    </a>
</nav>

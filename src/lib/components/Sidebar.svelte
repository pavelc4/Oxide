<script lang="ts">
    import { page } from '$app/stores';
    import ShapeBadge from '$lib/components/ShapeBadge.svelte';
    import type { MaterialShapeType } from '$lib/shapes/materialShapes';
    import { onMount } from 'svelte';

    let sidebarShape = $state<MaterialShapeType | 'rounded'>('cookie7');

    onMount(() => {
        if (typeof localStorage !== 'undefined') {
            const saved = localStorage.getItem('oxide:iconShape') as MaterialShapeType | 'rounded';
            if (saved) sidebarShape = saved;
        }
    });

    const navItems = [
        { href: '/', icon: 'devices', label: 'Devices' },
        { href: '/apps', icon: 'apps', label: 'Apps' },
        { href: '/files', icon: 'folder_open', label: 'Files' },
        { href: '/process-manager', icon: 'memory', label: 'Process' },
        { href: '/shell', icon: 'terminal', label: 'Shell' },
        { href: '/flasher', icon: 'bolt', label: 'Flasher' },
        { href: '/logcat', icon: 'subtitles', label: 'Logcat' },
        { href: '/audit', icon: 'history', label: 'Audit' }
    ];
</script>

<nav class="flex w-16 flex-col items-center justify-between py-6 shrink-0 h-screen bg-surface">
    <!-- Top Links -->
    <div class="flex flex-col items-center gap-3 overflow-y-auto min-h-0">
        <!-- Brand Logo Badge -->
        <a
            href="/"
            class="flex shrink-0 items-center justify-center transition-all hover:brightness-110 active:scale-95 no-underline mb-2"
            title="Oxide"
        >
            <ShapeBadge icon="science" shape="gem" size={40} iconSize={20} variant="primary" />
        </a>

        <!-- Sidebar Navigation Items -->
        {#each navItems as item}
          {@const isActive = $page.url.pathname === item.href}
          <a
            href={item.href}
            class="group flex flex-col items-center gap-1 no-underline"
            title={item.label}
          >
            {#if isActive}
                <ShapeBadge
                    icon={item.icon}
                    shape={sidebarShape}
                    size={36}
                    iconSize={18}
                    variant="primary"
                />
            {:else}
                <div class="flex h-9 w-9 items-center justify-center rounded-xl text-on-surface-variant/80 group-hover:bg-surface-container-high group-hover:text-on-surface transition-all">
                    <span class="material-symbols-outlined text-[20px]">{item.icon}</span>
                </div>
            {/if}
          </a>
        {/each}
    </div>

    <!-- Bottom Settings Link -->
    <a
        href="/settings"
        class="group flex flex-col items-center gap-1 no-underline mt-auto pt-2"
        title="Settings"
    >
        {#if $page.url.pathname === '/settings'}
            <ShapeBadge
                icon="settings"
                shape={sidebarShape}
                size={36}
                iconSize={18}
                variant="primary"
            />
        {:else}
            <div class="flex h-9 w-9 items-center justify-center rounded-xl text-on-surface-variant/80 group-hover:bg-surface-container-high group-hover:text-on-surface transition-all">
                <span class="material-symbols-outlined text-[20px]">settings</span>
            </div>
        {/if}
    </a>
</nav>

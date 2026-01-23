<script lang="ts">
  import { Activity, User, Settings, AlertTriangle, CheckCircle, Info } from '@lucide/svelte';
  import * as Card from '$lib/components/ui/card';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    loading: boolean;
  }

  let { loading }: Props = $props();

  interface ActivityItem {
    id: string;
    type: 'user' | 'system' | 'warning' | 'success' | 'info';
    message: string;
    timestamp: Date;
    user?: string;
  }

  // Mock activity data
  const activities: ActivityItem[] = [
    {
      id: '1',
      type: 'success',
      message: 'Container "api-gateway" successfully deployed',
      timestamp: new Date(Date.now() - 5 * 60000),
      user: 'admin',
    },
    {
      id: '2',
      type: 'user',
      message: 'User "john.doe" logged in',
      timestamp: new Date(Date.now() - 12 * 60000),
      user: 'john.doe',
    },
    {
      id: '3',
      type: 'warning',
      message: 'High CPU usage detected on production group',
      timestamp: new Date(Date.now() - 18 * 60000),
    },
    {
      id: '4',
      type: 'system',
      message: 'Automated backup completed successfully',
      timestamp: new Date(Date.now() - 25 * 60000),
    },
    {
      id: '5',
      type: 'info',
      message: 'System update available: v0.2.1',
      timestamp: new Date(Date.now() - 35 * 60000),
    },
    {
      id: '6',
      type: 'user',
      message: 'New resource group "staging" created',
      timestamp: new Date(Date.now() - 42 * 60000),
      user: 'admin',
    },
    {
      id: '7',
      type: 'success',
      message: 'SSL certificate renewed for api.csf-core.com',
      timestamp: new Date(Date.now() - 58 * 60000),
    },
  ];

  function getIcon(type: ActivityItem['type']) {
    switch (type) {
      case 'user':
        return User;
      case 'system':
        return Settings;
      case 'warning':
        return AlertTriangle;
      case 'success':
        return CheckCircle;
      case 'info':
        return Info;
      default:
        return Activity;
    }
  }

  function getIconClass(type: ActivityItem['type']): string {
    switch (type) {
      case 'user':
        return 'text-blue-500';
      case 'system':
        return 'text-gray-500';
      case 'warning':
        return 'text-yellow-500';
      case 'success':
        return 'text-green-500';
      case 'info':
        return 'text-purple-500';
      default:
        return 'text-muted-foreground';
    }
  }

  function formatTimestamp(date: Date): string {
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 60000); // Differenz in Minuten

    if (diff < 1) return 'Gerade eben';
    if (diff < 60) return `vor ${diff} Min`;
    const hours = Math.floor(diff / 60);
    if (hours < 24) return `vor ${hours} Std`;
    return `vor ${Math.floor(hours / 24)} Tagen`;
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Activity class="h-5 w-5" />
      Activity Feed
    </Card.Title>
    <Card.Description>Nutzeraktionen und Systemereignisse</Card.Description>
  </Card.Header>
  <Card.Content>
    {#if loading}
      <div class="space-y-4">
        {#each Array(5) as _}
          <Skeleton class="h-16 w-full" />
        {/each}
      </div>
    {:else}
      <div class="h-[400px] overflow-y-auto pr-4">
        <div class="space-y-4">
          {#each activities as activity}
            {@const Icon = getIcon(activity.type)}
            <div class="flex gap-3 items-start pb-4 border-b last:border-0">
              <div class="mt-0.5">
                <Icon class="h-4 w-4 {getIconClass(activity.type)}" />
              </div>
              <div class="flex-1 space-y-1">
                <p class="text-sm leading-tight">{activity.message}</p>
                <div class="flex items-center gap-2 text-xs text-muted-foreground">
                  <span>{formatTimestamp(activity.timestamp)}</span>
                  {#if activity.user}
                    <span>•</span>
                    <span class="font-medium">{activity.user}</span>
                  {/if}
                </div>
              </div>
              {#if activity.type === 'warning'}
                <Badge variant="outline" class="text-xs">Unresolved</Badge>
              {:else if activity.type === 'success'}
                <Badge variant="default" class="text-xs bg-green-500">Success</Badge>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- Activity Summary -->
      <div
        class="mt-4 pt-4 border-t flex items-center justify-between text-xs text-muted-foreground"
      >
        <span>Showing last 7 events</span>
        <a href="/activity" class="hover:text-foreground transition-colors"> View all → </a>
      </div>
    {/if}
  </Card.Content>
</Card.Root>

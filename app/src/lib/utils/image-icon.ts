const KNOWN_IMAGE_ICONS: Record<string, string> = {
    redis: 'logos:redis',
    postgres: 'logos:postgresql',
    postgresql: 'logos:postgresql',
    nginx: 'logos:nginx',
    mysql: 'logos:mysql',
    mariadb: 'logos:mariadb',
    mongo: 'logos:mongodb',
    mongodb: 'logos:mongodb',
    rabbitmq: 'logos:rabbitmq-icon',
    memcached: 'logos:memcached',
    traefik: 'logos:traefikproxy-icon',
    caddy: 'logos:caddy-icon',
    elasticsearch: 'logos:elasticsearch',
    grafana: 'logos:grafana',
    prometheus: 'logos:prometheus',
};

const DOCKER_FALLBACK_ICON = 'logos:docker-icon';

export function resolveImageIcon(image: string): string {
    const withoutTag = image.split('@')[0].split(':')[0];
    const segments = withoutTag.split('/');
    const bareName = segments[segments.length - 1].toLowerCase();

    return KNOWN_IMAGE_ICONS[bareName] ?? DOCKER_FALLBACK_ICON;
}

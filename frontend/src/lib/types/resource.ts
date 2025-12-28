export interface Resource {
  id: string;
  name: string;
  resource_type: string;
  description?: string;
  resource_group_id: string;
  resource_group_name: string;
  configuration?: Record<string, any>;
  status: 'pending' | 'running' | 'stopped' | 'error';
  created_by?: string;
  created_at: string;
  updated_at: string;
  tags?: Record<string, string>;
  container_id?: string;
  stack_name?: string;
}

export interface CreateResourceRequest {
  name: string;
  resource_type: string;
  description?: string;
  resource_group_id: string;
  configuration?: Record<string, any>;
  tags?: Record<string, string>;
}

export interface UpdateResourceRequest {
  name?: string;
  description?: string;
  configuration?: Record<string, any>;
  status?: 'pending' | 'running' | 'stopped' | 'error';
  tags?: Record<string, string>;
}

export interface DockerConfiguration {
  image: string;
  ports?: { container: number; host?: number }[];
  environment?: Record<string, string>;
  volumes?: { host: string; container: string }[];
  command?: string;
  restart_policy?: 'no' | 'on-failure' | 'always' | 'unless-stopped';
}

export interface DockerStackConfiguration {
  services: {
    name: string;
    image: string;
    ports?: { container: number; host?: number }[];
    environment?: Record<string, string>;
    volumes?: { host: string; container: string }[];
    depends_on?: string[];
    restart_policy?: 'no' | 'on-failure' | 'always' | 'unless-stopped';
  }[];
  networks?: {
    name: string;
    driver?: 'bridge' | 'host' | 'overlay';
  }[];
}

export interface ResourceTypeTemplate {
  type: string;
  name: string;
  description: string;
  icon: string;
  category: 'compute' | 'storage' | 'network' | 'database' | 'stack';
  configurationSchema?: Record<string, any>;
  template?: any;
  popular?: boolean;
}

export const RESOURCE_TYPES: ResourceTypeTemplate[] = [
  {
    type: 'docker-stack',
    name: 'Docker Stack',
    description: 'Mehrere zusammenhängende Docker Container. Perfekt für vollständige Anwendungen mit mehreren Services (z.B. Web + Datenbank + Cache).',
    icon: '📦',
    category: 'stack',
    popular: true,
    template: {
      services: [
        {
          name: 'web',
          image: 'nginx:alpine',
          ports: [{ container: 80, host: 8080 }],
          environment: {},
        }
      ]
    }
  },
];

// Vorgefertigte Stack Templates für den Marketplace
export const MARKETPLACE_TEMPLATES = [
  {
    id: 'wordpress',
    name: 'WordPress + MySQL',
    description: 'Vollständiger WordPress Stack mit MySQL Datenbank',
    icon: '📝',
    category: 'stack' as const,
    type: 'docker-stack',
    popular: true,
    configuration: {
      services: [
        {
          name: 'wordpress',
          image: 'wordpress:latest',
          ports: [{ container: 80, host: 8080 }],
          environment: {
            WORDPRESS_DB_HOST: 'db:3306',
            WORDPRESS_DB_USER: 'wordpress',
            WORDPRESS_DB_PASSWORD: 'wordpress',
            WORDPRESS_DB_NAME: 'wordpress',
          },
          depends_on: ['db'],
          restart_policy: 'always',
        },
        {
          name: 'db',
          image: 'mysql:8.0',
          environment: {
            MYSQL_DATABASE: 'wordpress',
            MYSQL_USER: 'wordpress',
            MYSQL_PASSWORD: 'wordpress',
            MYSQL_RANDOM_ROOT_PASSWORD: '1',
          },
          volumes: [{ host: './db_data', container: '/var/lib/mysql' }],
          restart_policy: 'always',
        }
      ]
    }
  },
  {
    id: 'mern-stack',
    name: 'MERN Stack',
    description: 'MongoDB + Express + React + Node.js Entwicklungsumgebung',
    icon: '🚀',
    category: 'stack' as const,
    type: 'docker-stack',
    popular: true,
    configuration: {
      services: [
        {
          name: 'mongodb',
          image: 'mongo:7',
          ports: [{ container: 27017, host: 27017 }],
          volumes: [{ host: './mongodb_data', container: '/data/db' }],
          restart_policy: 'always',
        },
        {
          name: 'backend',
          image: 'node:20-alpine',
          ports: [{ container: 5000, host: 5000 }],
          environment: {
            MONGODB_URI: 'mongodb://mongodb:27017/app',
            NODE_ENV: 'development',
          },
          depends_on: ['mongodb'],
          restart_policy: 'always',
        },
        {
          name: 'frontend',
          image: 'node:20-alpine',
          ports: [{ container: 3000, host: 3000 }],
          environment: {
            REACT_APP_API_URL: 'http://localhost:5000',
          },
          restart_policy: 'always',
        }
      ]
    }
  },
  {
    id: 'nginx-postgres',
    name: 'NGINX + PostgreSQL',
    description: 'Web Server mit PostgreSQL Datenbank',
    icon: '🌐',
    category: 'stack' as const,
    type: 'docker-stack',
    configuration: {
      services: [
        {
          name: 'nginx',
          image: 'nginx:alpine',
          ports: [{ container: 80, host: 8080 }],
          volumes: [{ host: './html', container: '/usr/share/nginx/html' }],
          depends_on: ['postgres'],
          restart_policy: 'always',
        },
        {
          name: 'postgres',
          image: 'postgres:16',
          environment: {
            POSTGRES_USER: 'admin',
            POSTGRES_PASSWORD: 'admin',
            POSTGRES_DB: 'appdb',
          },
          volumes: [{ host: './postgres_data', container: '/var/lib/postgresql/data' }],
          restart_policy: 'always',
        }
      ]
    }
  },
];

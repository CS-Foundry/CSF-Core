export interface LogEntry {
	level: 'debug' | 'info' | 'warn' | 'error';
	message: string;
	data?: any;
	timestamp: string;
	context?: string;
}

class Logger {
	private context: string;

	constructor(context: string = 'App') {
		this.context = context;
	}

	private log(level: LogEntry['level'], message: string, data?: any) {
		const entry: LogEntry = {
			level,
			message,
			data,
			timestamp: new Date().toISOString(),
			context: this.context
		};

		const logMessage = `[${entry.timestamp}] [${level.toUpperCase()}] [${entry.context}] ${message}`;

		switch (level) {
			case 'debug':
				console.debug(logMessage, data || '');
				break;
			case 'info':
				console.info(logMessage, data || '');
				break;
			case 'warn':
				console.warn(logMessage, data || '');
				break;
			case 'error':
				console.error(logMessage, data || '');
				break;
		}
	}

	debug(message: string, data?: any) {
		this.log('debug', message, data);
	}

	info(message: string, data?: any) {
		this.log('info', message, data);
	}

	warn(message: string, data?: any) {
		this.log('warn', message, data);
	}

	error(message: string, data?: any) {
		this.log('error', message, data);
	}
}

// Create a default logger instance
export const logger = new Logger('FinanceVault');

// Factory function to create context-specific loggers
export function createLogger(context: string): Logger {
	return new Logger(context);
}
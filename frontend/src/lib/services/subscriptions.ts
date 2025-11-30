import { ApiClient } from './api-client';

export interface Subscription {
	id: string;
	user_id: string;
	name: string;
	amount: number;
	billing_cycle: string;
	next_billing_date: string;
	category: string;
	is_active: boolean;
}

export interface CreateSubscriptionRequest {
	name: string;
	amount: number;
	billing_cycle: string;
	next_billing_date: string;
	category: string;
}

export interface UpdateSubscriptionRequest {
	name?: string;
	amount?: number;
	billing_cycle?: string;
	next_billing_date?: string;
	category?: string;
	is_active?: boolean;
}

export class SubscriptionService {
	static async getSubscriptions(token: string): Promise<Subscription[]> {
		const response = await ApiClient.get('/subscriptions', token);

		if (!response.ok) {
			throw new Error('Failed to fetch subscriptions');
		}

		return await response.json();
	}

	static async getSubscription(id: string, token: string): Promise<Subscription> {
		const response = await ApiClient.get(`/subscriptions/${id}`, token);

		if (!response.ok) {
			if (response.status === 404) {
				throw new Error('Subscription not found');
			}
			throw new Error('Failed to fetch subscription');
		}

		return await response.json();
	}

	static async createSubscription(
		subscription: CreateSubscriptionRequest,
		token: string
	): Promise<Subscription> {
		const response = await ApiClient.post('/subscriptions', subscription, token);

		if (!response.ok) {
			throw new Error('Failed to create subscription');
		}

		return await response.json();
	}

	static async updateSubscription(
		id: string,
		subscription: UpdateSubscriptionRequest,
		token: string
	): Promise<Subscription> {
		const response = await ApiClient.put(`/subscriptions/${id}`, subscription, token);

		if (!response.ok) {
			if (response.status === 404) {
				throw new Error('Subscription not found');
			}
			throw new Error('Failed to update subscription');
		}

		return await response.json();
	}

	static async deleteSubscription(id: string, token: string): Promise<void> {
		const response = await ApiClient.delete(`/subscriptions/${id}`, token);

		if (!response.ok) {
			if (response.status === 404) {
				throw new Error('Subscription not found');
			}
			throw new Error('Failed to delete subscription');
		}
	}
}

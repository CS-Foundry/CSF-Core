import { ApiClient } from './api-client';
import type { Budget, CreateBudgetRequest, UpdateBudgetRequest, BudgetOverview } from '$lib/types';

export class BudgetService {
  static async getBudgets(token: string): Promise<Budget[]> {
    const response = await ApiClient.get('/budgets', token);

    if (!response.ok) {
      throw new Error('Failed to fetch budgets');
    }

    return await response.json();
  }

  static async getBudgetByMonth(month: string, token: string): Promise<Budget> {
    const response = await ApiClient.get(`/budgets/${month}`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Budget not found for this month');
      }
      throw new Error('Failed to fetch budget');
    }

    return await response.json();
  }

  static async getBudgetOverview(month: string, token: string): Promise<BudgetOverview> {
    const response = await ApiClient.get(`/budgets/${month}/overview`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Budget not found for this month');
      }
      throw new Error('Failed to fetch budget overview');
    }

    return await response.json();
  }

  static async createBudget(budget: CreateBudgetRequest, token: string): Promise<Budget> {
    const response = await ApiClient.post('/budgets', budget, token);

    if (!response.ok) {
      throw new Error('Failed to create budget');
    }

    return await response.json();
  }

  static async updateBudget(
    month: string,
    budget: UpdateBudgetRequest,
    token: string
  ): Promise<Budget> {
    const response = await ApiClient.put(`/budgets/${month}`, budget, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Budget not found');
      }
      throw new Error('Failed to update budget');
    }

    return await response.json();
  }

  static async deleteBudget(month: string, token: string): Promise<void> {
    const response = await ApiClient.delete(`/budgets/${month}`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Budget not found');
      }
      throw new Error('Failed to delete budget');
    }
  }
}

import { ApiClient } from './api-client';

export interface Expense {
  id: string;
  user_id: string;
  description: string;
  amount: number;
  date: string;
  category: string;
}

export interface CreateExpenseRequest {
  description: string;
  amount: number;
  date: string;
  category: string;
}

export interface UpdateExpenseRequest {
  description?: string;
  amount?: number;
  date?: string;
  category?: string;
}

export class ExpenseService {
  static async getExpenses(token: string): Promise<Expense[]> {
    const response = await ApiClient.get('/expenses', token);

    if (!response.ok) {
      throw new Error('Failed to fetch expenses');
    }

    return await response.json();
  }

  static async getExpense(id: string, token: string): Promise<Expense> {
    const response = await ApiClient.get(`/expenses/${id}`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Expense not found');
      }
      throw new Error('Failed to fetch expense');
    }

    return await response.json();
  }

  static async createExpense(expense: CreateExpenseRequest, token: string): Promise<Expense> {
    const response = await ApiClient.post('/expenses', expense, token);

    if (!response.ok) {
      throw new Error('Failed to create expense');
    }

    return await response.json();
  }

  static async updateExpense(
    id: string,
    expense: UpdateExpenseRequest,
    token: string
  ): Promise<Expense> {
    const response = await ApiClient.put(`/expenses/${id}`, expense, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Expense not found');
      }
      throw new Error('Failed to update expense');
    }

    return await response.json();
  }

  static async deleteExpense(id: string, token: string): Promise<void> {
    const response = await ApiClient.delete(`/expenses/${id}`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Expense not found');
      }
      throw new Error('Failed to delete expense');
    }
  }
}

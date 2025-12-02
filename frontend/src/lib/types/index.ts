// Re-export types from services
export type { Expense, CreateExpenseRequest, UpdateExpenseRequest } from '../services/expenses';
export type { Subscription, CreateSubscriptionRequest, UpdateSubscriptionRequest } from '../services/subscriptions';

// Budget types
export interface Budget {
	id: string;
	user_id: string;
	month: string; // YYYY-MM format
	total_budget: number;
	categories: BudgetCategory[];
	created_at: string;
	updated_at: string;
}

export interface BudgetCategory {
	category: string;
	allocated_amount: number;
	spent_amount: number;
}

export interface CreateBudgetRequest {
	month: string;
	total_budget: number;
	categories: {
		category: string;
		allocated_amount: number;
	}[];
}

export interface UpdateBudgetRequest {
	total_budget?: number;
	categories?: {
		category: string;
		allocated_amount: number;
	}[];
}

export interface BudgetOverview {
	budget: Budget;
	total_spent: number;
	remaining: number;
	percentage_used: number;
	categories: {
		category: string;
		allocated: number;
		spent: number;
		remaining: number;
		percentage_used: number;
	}[];
}

// Invoice types
export interface Invoice {
	id: string;
	user_id: string;
	month: string; // YYYY-MM format
	description: string;
	amount: number;
	date: string;
	category: string;
	file_url?: string;
	verified: boolean;
	created_at: string;
	updated_at: string;
}

export interface ManualEntry {
	id: string;
	user_id: string;
	month: string;
	description: string;
	amount: number;
	date: string;
	category: string;
	matched: boolean;
	matched_invoice_id?: string;
	created_at: string;
	updated_at: string;
}

export interface CreateInvoiceRequest {
	month: string;
	description: string;
	amount: number;
	date: string;
	category: string;
	file?: File;
}

export interface CreateManualEntryRequest {
	month: string;
	description: string;
	amount: number;
	date: string;
	category: string;
}

export interface InvoiceMatch {
	invoice_id: string;
	manual_entry_id: string;
	match_confidence: number;
	matched_at: string;
}

export interface InvoiceOverview {
	month: string;
	invoices: Invoice[];
	manual_entries: ManualEntry[];
	matches: InvoiceMatch[];
	total_invoices: number;
	total_manual: number;
	matched_count: number;
	unmatched_invoices: number;
	unmatched_manual: number;
}

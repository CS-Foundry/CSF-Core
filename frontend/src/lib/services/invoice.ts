import { ApiClient } from './api-client';
import type {
  Invoice,
  ManualEntry,
  CreateInvoiceRequest,
  CreateManualEntryRequest,
  InvoiceOverview,
} from '$lib/types';

export class InvoiceService {
  static async getInvoices(month: string, token: string): Promise<Invoice[]> {
    const response = await ApiClient.get(`/invoices?month=${month}`, token);

    if (!response.ok) {
      throw new Error('Failed to fetch invoices');
    }

    return await response.json();
  }

  static async getInvoice(id: string, token: string): Promise<Invoice> {
    const response = await ApiClient.get(`/invoices/${id}`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Invoice not found');
      }
      throw new Error('Failed to fetch invoice');
    }

    return await response.json();
  }

  static async createInvoice(invoice: CreateInvoiceRequest, token: string): Promise<Invoice> {
    const formData = new FormData();
    formData.append('month', invoice.month);
    formData.append('description', invoice.description);
    formData.append('amount', invoice.amount.toString());
    formData.append('date', invoice.date);
    formData.append('category', invoice.category);

    if (invoice.file) {
      formData.append('file', invoice.file);
    }

    const response = await ApiClient.fetch(
      '/invoices',
      {
        method: 'POST',
        body: formData,
      },
      token
    );

    if (!response.ok) {
      throw new Error('Failed to create invoice');
    }

    return await response.json();
  }

  static async deleteInvoice(id: string, token: string): Promise<void> {
    const response = await ApiClient.delete(`/invoices/${id}`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Invoice not found');
      }
      throw new Error('Failed to delete invoice');
    }
  }

  static async getManualEntries(month: string, token: string): Promise<ManualEntry[]> {
    const response = await ApiClient.get(`/manual-entries?month=${month}`, token);

    if (!response.ok) {
      throw new Error('Failed to fetch manual entries');
    }

    return await response.json();
  }

  static async createManualEntry(
    entry: CreateManualEntryRequest,
    token: string
  ): Promise<ManualEntry> {
    const response = await ApiClient.post('/manual-entries', entry, token);

    if (!response.ok) {
      throw new Error('Failed to create manual entry');
    }

    return await response.json();
  }

  static async deleteManualEntry(id: string, token: string): Promise<void> {
    const response = await ApiClient.delete(`/manual-entries/${id}`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Manual entry not found');
      }
      throw new Error('Failed to delete manual entry');
    }
  }

  static async getInvoiceOverview(month: string, token: string): Promise<InvoiceOverview> {
    const response = await ApiClient.get(`/invoices/${month}/overview`, token);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('No data found for this month');
      }
      throw new Error('Failed to fetch invoice overview');
    }

    return await response.json();
  }

  static async matchInvoices(month: string, token: string): Promise<InvoiceOverview> {
    const response = await ApiClient.post(`/invoices/${month}/match`, {}, token);

    if (!response.ok) {
      throw new Error('Failed to match invoices');
    }

    return await response.json();
  }
}

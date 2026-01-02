import { ApiClient } from './api-client';
import { logger } from '$lib/utils/logger';

export interface Setup2FAResponse {
  secret: string;
  qr_code: string;
}

export class SettingsService {
  /**
   * Setup 2FA - Generate secret and QR code
   */
  static async setup2FA(): Promise<Setup2FAResponse> {
    logger.info('Setting up 2FA');

    const response = await ApiClient.post('/2fa/setup');

    if (!response.ok) {
      throw new Error(`Failed to setup 2FA: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Enable 2FA with verification code
   */
  static async enable2FA(code: string): Promise<void> {
    logger.info('Enabling 2FA');

    const response = await ApiClient.post('/2fa/enable', { code });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to enable 2FA');
    }
  }

  /**
   * Disable 2FA with verification code
   */
  static async disable2FA(code: string): Promise<void> {
    logger.info('Disabling 2FA');

    const response = await ApiClient.post('/2fa/disable', { code });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to disable 2FA');
    }
  }

  /**
   * Change user password
   */
  static async changePassword(oldPassword: string, newPassword: string): Promise<void> {
    logger.info('Changing password');

    const response = await ApiClient.post('/change-password', {
      old_password: oldPassword,
      new_password: newPassword,
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to change password');
    }
  }

  /**
   * Change user email
   */
  static async changeEmail(newEmail: string): Promise<void> {
    logger.info('Changing email');

    const response = await ApiClient.post('/change-email', {
      new_email: newEmail,
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to change email');
    }
  }

  /**
   * Get user profile with 2FA status
   */
  static async getProfile(): Promise<{
    id: string;
    username: string;
    email?: string;
    two_factor_enabled: boolean;
    force_password_change: boolean;
  }> {
    logger.info('Fetching user profile');

    const response = await ApiClient.get('/profile');

    if (!response.ok) {
      throw new Error(`Failed to get profile: ${response.statusText}`);
    }

    return response.json();
  }
}

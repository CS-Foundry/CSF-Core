import type { PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ parent }) => {
  const { user } = await parent();

  if (!user) {
    throw redirect(302, '/signin');
  }

  // In a real implementation, you would check user's role/permissions here
  // For now, we allow access if user is authenticated
  // You can add RBAC check by calling the backend API to verify permissions

  return {
    user,
  };
};

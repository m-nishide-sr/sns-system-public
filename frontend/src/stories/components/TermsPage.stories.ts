/**
 * 利用規約ページの表示確認用 Storybook ストーリー。
 */
import type { Meta, StoryObj } from '@storybook/nextjs-vite';
import { TermsPageView } from '@/components/pages/TermsPage';

const meta = {
  title: 'Pages/TermsPage',
  component: TermsPageView,
  parameters: {
    layout: 'fullscreen',
  },
  args: {
    onNavigate: () => {},
  },
} satisfies Meta<typeof TermsPageView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  name: '利用規約',
};

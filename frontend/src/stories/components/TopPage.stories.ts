import type { Meta, StoryObj } from '@storybook/nextjs-vite';
import { TopPageView } from '@/components/pages/TopPage';

const meta = {
  title: 'Pages/TopPage',
  component: TopPageView,
  parameters: {
    layout: 'fullscreen',
  },
  args: {
    isAuthenticated: false,
    onNavigate: () => {},
  },
} satisfies Meta<typeof TopPageView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NotAuthenticated: Story = {
  name: '未ログイン',
};

export const Authenticated: Story = {
  name: 'ログイン中',
  args: {
    isAuthenticated: true,
  },
};

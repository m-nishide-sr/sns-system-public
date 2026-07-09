import type { Meta, StoryObj } from '@storybook/nextjs-vite';
import { MenuView } from '@/components/Menu';

const meta = {
  title: 'Components/Menu',
  component: MenuView,
  parameters: {
    layout: 'fullscreen',
  },
  args: {
    currentRoute: '/',
    isAuthenticated: false,
    isOpen: true,
    onClose: () => {},
    onNavigate: () => {},
  },
} satisfies Meta<typeof MenuView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Open: Story = {
  name: 'メニュー開',
};

export const Closed: Story = {
  name: 'メニュー閉',
  args: {
    isOpen: false,
  },
};

export const Authenticated: Story = {
  name: 'ログイン中',
  args: {
    isAuthenticated: true,
    currentRoute: '/chat',
  },
};

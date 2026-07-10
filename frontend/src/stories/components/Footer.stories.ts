import type { Meta, StoryObj } from '@storybook/nextjs-vite';
import { FooterView } from '@/components/Footer';

const meta = {
  title: 'Components/Footer',
  component: FooterView,
  parameters: {
    layout: 'fullscreen',
  },
  args: {
    year: 2025,
    ownerName: 'SNS System',
  },
} satisfies Meta<typeof FooterView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  name: 'デフォルト',
};

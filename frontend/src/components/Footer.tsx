type FooterViewProps = {
  year: number;
  ownerName: string;
};

export function FooterView({ year, ownerName }: FooterViewProps) {
  return (
    <footer className="border-t border-slate-200/80 bg-white/80">
      <div className="mx-auto max-w-6xl px-4 py-6 text-center text-sm text-slate-500 sm:px-6 lg:px-8">
        © {year} {ownerName}
      </div>
    </footer>
  );
}

export function Footer() {
  return <FooterView year={new Date().getFullYear()} ownerName="SNS System" />;
}

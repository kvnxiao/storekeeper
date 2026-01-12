interface SectionProps {
  title: string;
  description?: string;
  children: React.ReactNode;
}

export const Section: React.FC<SectionProps> = ({
  title,
  description,
  children,
}) => (
  <section className="rounded-lg border border-zinc-200 bg-white p-4 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
    <h2 className="mb-1 text-lg font-semibold text-zinc-950 dark:text-white">
      {title}
    </h2>
    {description && (
      <p className="mb-4 text-sm text-zinc-500 dark:text-zinc-400">
        {description}
      </p>
    )}
    <div className="space-y-4">{children}</div>
  </section>
);

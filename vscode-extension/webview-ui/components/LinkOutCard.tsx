import React from 'react';
import { Button, Card, CardContent } from './primitives';
import { SectionHeader } from './SectionHeader';

interface LinkOutCardProps {
  id: string;
  title: string;
  description: string;
  onOpen: () => void;
}

export function LinkOutCard({ id, title, description, onOpen }: LinkOutCardProps) {
  return (
    <div className="op-mb-4">
      <SectionHeader id={id} title={title} />
      <Card>
        <CardContent>
          <div className="op-row op-space-between op-gap-2">
            <p className="op-body2 op-text-secondary">{description}</p>
            <Button variant="outlined" onClick={onOpen}>
              Open in Operator UI →
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

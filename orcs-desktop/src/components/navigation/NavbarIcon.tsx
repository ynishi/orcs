import { Box, Tooltip, Badge } from '@mantine/core';
import { ReactNode } from 'react';

interface NavbarIconProps {
  icon: ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
  badge?: number;
}

export function NavbarIcon({ icon, label, active, onClick, badge }: NavbarIconProps) {
  return (
    <Tooltip label={label} position="right" withArrow>
      <Box
        onClick={onClick}
        style={{
          width: 48,
          height: 48,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          cursor: 'pointer',
          position: 'relative',
          backgroundColor: active ? 'var(--mantine-color-blue-light)' : 'transparent',
          borderLeft: active ? '3px solid var(--mantine-color-blue-6)' : '3px solid transparent',
          transition: 'all 0.2s ease',
        }}
        onMouseEnter={(e) => {
          if (!active) {
            e.currentTarget.style.backgroundColor = 'var(--mantine-color-gray-1)';
          }
        }}
        onMouseLeave={(e) => {
          if (!active) {
            e.currentTarget.style.backgroundColor = 'transparent';
          }
        }}
      >
        <Box style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', color: active ? 'var(--mantine-color-blue-6)' : 'var(--mantine-color-gray-7)' }}>
          {icon}
        </Box>
        {badge !== undefined && badge > 0 && (
          <Badge
            size="xs"
            variant="filled"
            color="blue"
            style={{
              position: 'absolute',
              top: 4,
              right: 4,
              minWidth: 16,
              height: 16,
              padding: '0 4px',
            }}
          >
            {badge}
          </Badge>
        )}
      </Box>
    </Tooltip>
  );
}

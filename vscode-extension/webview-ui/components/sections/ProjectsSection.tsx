import React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';
import Link from '@mui/material/Link';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import Typography from '@mui/material/Typography';
import { SectionHeader } from '../SectionHeader';
import type { ProjectSummary } from '../../types/messages';

interface ProjectsSectionProps {
  projects: ProjectSummary[];
  loading: boolean;
  error: string | null;
  onAssess: (name: string) => void;
  onOpenProject: (path: string) => void;
  onRefresh: () => void;
}

export function ProjectsSection({
  projects,
  loading,
  error,
  onAssess,
  onOpenProject,
  onRefresh,
}: ProjectsSectionProps) {
  return (
    <Box sx={{ mb: 4 }}>
      <SectionHeader id="section-projects" title="Operator Managed Projects" />
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
        <Typography color="text.secondary">
          Projects discovered by the Operator API with analysis data from ASSESS tickets.
        </Typography>
        <Button size="small" variant="outlined" onClick={onRefresh} disabled={loading}>
          Refresh
        </Button>
      </Box>

      {loading && (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, py: 2 }}>
          <CircularProgress size={20} />
          <Typography variant="body2" color="text.secondary">Loading projects...</Typography>
        </Box>
      )}

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>
      )}

      {!loading && !error && projects.length === 0 && (
        <Typography variant="body2" color="text.secondary">
          No projects found. Ensure the Operator API is running and projects are configured.
        </Typography>
      )}

      {!loading && projects.length > 0 && (
        <TableContainer>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Project</TableCell>
                <TableCell>Kind &amp; Stack</TableCell>
                <TableCell>Detections</TableCell>
                <TableCell>Config</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {projects.map((project) => (
                <TableRow key={project.project_name} sx={{ '&:last-child td': { borderBottom: 0 } }}>
                  <TableCell>
                    <Link
                      component="button"
                      variant="body2"
                      onClick={() => onOpenProject(project.project_path)}
                      sx={{ fontWeight: 600, textAlign: 'left' }}
                    >
                      {project.project_name}
                    </Link>
                    {!project.exists && (
                      <Typography variant="caption" color="error" sx={{ display: 'block' }}>
                        Directory not found
                      </Typography>
                    )}
                  </TableCell>
                  <TableCell>
                    <Stack direction="row" spacing={0.5} flexWrap="wrap" useFlexGap>
                      {project.kind && (
                        <Chip
                          label={project.kind}
                          size="small"
                          color="primary"
                          variant="filled"
                          sx={{ fontWeight: 600 }}
                        />
                      )}
                      {project.languages.map((lang) => (
                        <Chip key={lang} label={lang} size="small" variant="outlined" />
                      ))}
                      {project.frameworks.map((fw) => (
                        <Chip key={fw} label={fw} size="small" variant="outlined" color="secondary" />
                      ))}
                      {project.databases.map((db) => (
                        <Chip key={db} label={db} size="small" variant="outlined" color="info" />
                      ))}
                    </Stack>
                  </TableCell>
                  <TableCell>
                    <Stack direction="row" spacing={0.5} flexWrap="wrap" useFlexGap>
                      {project.has_docker && <Chip label="Docker" size="small" variant="outlined" />}
                      {project.has_tests && <Chip label="Tests" size="small" variant="outlined" />}
                      {project.has_catalog_info && <Chip label="Catalog" size="small" variant="outlined" />}
                      {project.has_project_context && <Chip label="Context" size="small" variant="outlined" />}
                    </Stack>
                  </TableCell>
                  <TableCell>
                    <Typography variant="caption" sx={{ whiteSpace: 'nowrap' }}>
                      {project.ports.length > 0 && `${project.ports.length} ports`}
                      {project.ports.length > 0 && project.env_var_count > 0 && ' · '}
                      {project.env_var_count > 0 && `${project.env_var_count} env`}
                      {(project.ports.length > 0 || project.env_var_count > 0) && project.entry_point_count > 0 && ' · '}
                      {project.entry_point_count > 0 && `${project.entry_point_count} entry`}
                      {project.ports.length === 0 && project.env_var_count === 0 && project.entry_point_count === 0 && '—'}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    <Button
                      size="small"
                      variant="outlined"
                      onClick={() => onAssess(project.project_name)}
                      disabled={!project.exists}
                    >
                      {project.has_project_context ? 'Re-assess' : 'Assess'}
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Box>
  );
}

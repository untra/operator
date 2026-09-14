{{- define "operator.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "operator.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{- define "operator.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "operator.selectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}

{{- define "operator.selectorLabels" -}}
app.kubernetes.io/name: {{ include "operator.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "operator.serviceAccountName" -}}
{{- default (include "operator.fullname" .) .Values.serviceAccount.name }}
{{- end }}

{{- define "operator.validatePodExtensions" -}}
{{- $ownedVolumes := dict "workspace" true "home" true "tmp" true "bootstrap" true -}}
{{- $volumeNames := dict -}}
{{- range .Values.extraVolumes -}}
  {{- if hasKey $ownedVolumes .name -}}
    {{- fail (printf "extraVolumes name %q conflicts with a chart-owned volume" .name) -}}
  {{- end -}}
  {{- if hasKey $volumeNames .name -}}
    {{- fail (printf "extraVolumes contains duplicate name %q" .name) -}}
  {{- end -}}
  {{- $_ := set $volumeNames .name true -}}
{{- end -}}
{{- $ownedMountPaths := dict "/op" true "/home/operator" true "/tmp" true "/run/secrets/operator-bootstrap" true -}}
{{- $mountPaths := dict -}}
{{- range .Values.extraVolumeMounts -}}
  {{- if hasKey $ownedMountPaths .mountPath -}}
    {{- fail (printf "extraVolumeMounts mountPath %q conflicts with a chart-owned mount" .mountPath) -}}
  {{- end -}}
  {{- if hasKey $mountPaths .mountPath -}}
    {{- fail (printf "extraVolumeMounts contains duplicate mountPath %q" .mountPath) -}}
  {{- end -}}
  {{- $_ := set $mountPaths .mountPath true -}}
{{- end -}}
{{- $shutdownBudget := add .Values.shutdownDrainSeconds .Values.shutdownCleanupSeconds -}}
{{- if le (int .Values.terminationGracePeriodSeconds) (int $shutdownBudget) -}}
  {{- fail "terminationGracePeriodSeconds must be greater than shutdownDrainSeconds + shutdownCleanupSeconds" -}}
{{- end -}}
{{- end -}}

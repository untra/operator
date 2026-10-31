{{- define "operator.telemetryEnv" -}}
{{- if or .Values.telemetry.enabled .Values.telemetry.export.endpoint }}
- name: OPERATOR_LOGGING__FORMAT
  value: json
- name: OPERATOR_TELEMETRY__ENABLED
  value: {{ .Values.telemetry.enabled | quote }}
- name: OPERATOR_TELEMETRY__ENABLE_REDACTION
  value: {{ .Values.telemetry.enableRedaction | quote }}
- name: OPERATOR_TELEMETRY__INGEST__INJECT_AGENT_ENV
  value: {{ .Values.telemetry.injectAgentEnv | quote }}
- name: OPERATOR_TELEMETRY__WINDOW__MAX_SESSIONS
  value: {{ .Values.telemetry.window.maxSessions | quote }}
- name: OPERATOR_TELEMETRY__WINDOW__RECORDS_PER_SESSION
  value: {{ .Values.telemetry.window.recordsPerSession | quote }}
- name: OPERATOR_TELEMETRY__WINDOW__GRACE_SECS
  value: {{ .Values.telemetry.window.graceSeconds | quote }}
{{- with .Values.telemetry.export.endpoint }}
- name: OTEL_EXPORTER_OTLP_ENDPOINT
  value: {{ . | quote }}
{{- end }}
- name: OTEL_EXPORTER_OTLP_PROTOCOL
  value: {{ .Values.telemetry.export.protocol | quote }}
- name: OTEL_EXPORTER_OTLP_COMPRESSION
  value: {{ .Values.telemetry.export.compression | quote }}
- name: OTEL_EXPORTER_OTLP_TIMEOUT
  value: {{ .Values.telemetry.export.timeoutMillis | quote }}
- name: OTEL_SERVICE_NAME
  value: {{ .Values.telemetry.export.serviceName | quote }}
- name: POD_NAME
  valueFrom:
    fieldRef:
      fieldPath: metadata.name
- name: POD_NAMESPACE
  valueFrom:
    fieldRef:
      fieldPath: metadata.namespace
- name: OTEL_RESOURCE_ATTRIBUTES
  value: "k8s.pod.name=$(POD_NAME),k8s.namespace.name=$(POD_NAMESPACE)"
{{- with .Values.telemetry.export.headers.existingSecret }}
- name: OPERATOR_OTLP_HEADERS
  valueFrom:
    secretKeyRef:
      name: {{ . | quote }}
      key: {{ $.Values.telemetry.export.headers.key | quote }}
- name: OPERATOR_TELEMETRY__EXPORT__HEADERS_ENV
  value: OPERATOR_OTLP_HEADERS
{{- end }}
{{- end }}
{{- end }}

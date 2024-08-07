{{- define "helpers.list-env-variables" -}}
envFrom:
{{- range .Values.secrets }}
- secretRef:
    name: {{ . }}
{{- end }}
{{- end -}}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "bonsol.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

###########################################################
## prover node
###########################################################

{{/*
Expand the name of the chart.
*/}}
{{- define "bonsol-provernode.name" -}}
{{- default .Chart.Name .Values.provernode.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "bonsol-provernode.fullname" -}}
{{- if .Values.provernode.fullnameOverride }}
{{- .Values.provernode.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.provernode.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "bonsol-provernode.labels" -}}
helm.sh/chart: {{ include "bonsol.chart" . }}
{{ include "bonsol-provernode.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "bonsol-provernode.selectorLabels" -}}
app.kubernetes.io/name: {{ include "bonsol-provernode.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

###########################################################
## tester
###########################################################


{{/*
Expand the name of the chart.
*/}}
{{- define "bonsol-tester.name" -}}
{{- default (printf "%s-test" .Chart.Name) .Values.tester.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "bonsol-tester.fullname" -}}
{{- if .Values.tester.fullnameOverride }}
{{- .Values.tester.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.tester.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s-test" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "bonsol-tester.labels" -}}
helm.sh/chart: {{ include "bonsol.chart" . }}
{{ include "bonsol-tester.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "bonsol-tester.selectorLabels" -}}
app.kubernetes.io/name: {{ include "bonsol-tester.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Path to deployment manifest
*/}}
{{- define "bonsol-tester.manifestPath" -}}
/input_files/simple_manifest.json
{{- end }}

{{/*
Path to execution request
*/}}
{{- define "bonsol-tester.executionReqPath" -}}
/input_files/simple_execution_request.json
{{- end }}


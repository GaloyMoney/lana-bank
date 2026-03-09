---
id: documents
title: Gestión de Documentos
sidebar_position: 3
---

# Gestión de Documentos del Cliente

Este documento describe el sistema de gestión de documentos para clientes, incluyendo carga, almacenamiento y recuperación de documentos.

## Tipos de Documentos

### Documentos de Identidad

| Tipo | Descripción | Requerido para KYC |
|------|-------------|--------------------|
| Identificación oficial | DNI, pasaporte, licencia | Sí |
| Selfie | Foto del cliente | Sí |
| Comprobante de domicilio | Recibo de servicios | Según configuración |

### Documentos Corporativos

| Tipo | Descripción | Aplica a |
|------|-------------|----------|
| Escritura constitutiva | Documento de incorporación | Empresas |
| Poder notarial | Representación legal | Empresas |
| Estados financieros | Información financiera | Empresas |

## Arquitectura de Almacenamiento

```
┌─────────────────────────────────────────────────────────────────┐
│                    Panel de Administración                      │
│                    (Carga de documentos)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    API GraphQL                                  │
│               (Mutation: uploadDocument)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                 document-storage                                │
│           (Servicio de almacenamiento)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              Google Cloud Storage / S3                          │
│            (Almacenamiento de archivos)                         │
└─────────────────────────────────────────────────────────────────┘
```

## Operaciones de Documentos

### Subir Documento

1. Navegar al detalle del cliente
2. Seleccionar **Documentos** > **Subir**
3. Seleccionar tipo de documento
4. Arrastrar o seleccionar archivo
5. Confirmar carga

### Formatos Soportados

| Formato | Extensión | Tamaño Máximo |
|---------|-----------|---------------|
| PDF | .pdf | 10 MB |
| Imagen | .jpg, .png | 5 MB |
| Documento | .doc, .docx | 10 MB |

### Via API GraphQL

```graphql
mutation UploadDocument($input: DocumentUploadInput!) {
  documentUpload(input: $input) {
    document {
      id
      filename
      status
      createdAt
    }
  }
}
```

El input incluye:
- `customerId`: ID del cliente
- `documentType`: Tipo de documento
- `file`: Archivo (multipart upload)

## Estados del Documento

| Estado | Descripción |
|--------|-------------|
| PENDING | Cargado, pendiente de revisión |
| APPROVED | Documento validado |
| REJECTED | Documento rechazado |
| EXPIRED | Documento vencido |

## Flujo de Aprobación de Documentos

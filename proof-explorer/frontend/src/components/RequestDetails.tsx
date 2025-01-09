import { useParams, Link } from 'react-router-dom';
import { useQuery } from 'react-query';
import {
  Box,
  Stack,
  Heading,
  Text,
  Badge,
  SimpleGrid,
  GridItem,
  Spinner,
  Button,
  Code,
  useClipboard,
  IconButton,
  Flex,
} from '@chakra-ui/react';
import { ArrowBackIcon } from '@chakra-ui/icons';
import { getRequest } from '../services/api';
import { RequestStatus, ChainStatus } from '../types';

const RequestDetails = () => {
  const { id } = useParams<{ id: string }>();
  const { data: request, isLoading, error } = useQuery(['request', id], () => getRequest(id!));
  const { hasCopied, onCopy } = useClipboard(request?.executionAddress || '');

  const getStatusBadge = (status: RequestStatus) => {
    const colors = {
      [RequestStatus.PENDING]: 'yellow',
      [RequestStatus.IN_PROGRESS]: 'blue',
      [RequestStatus.COMPLETED]: 'green',
      [RequestStatus.FAILED]: 'red',
    };
    return <Badge colorScheme={colors[status]}>{status}</Badge>;
  };

  const getChainStatusBadge = (status: ChainStatus) => {
    const colors = {
      [ChainStatus.UNCLAIMED]: 'gray',
      [ChainStatus.CLAIMED]: 'purple',
      [ChainStatus.EXECUTED]: 'teal',
      [ChainStatus.POSTED]: 'green',
    };
    return <Badge colorScheme={colors[status]}>{status}</Badge>;
  };

  if (isLoading) {
    return (
      <Box textAlign="center" py={10}>
        <Spinner size="xl" />
      </Box>
    );
  }

  if (error || !request) {
    return (
      <Box textAlign="center" py={10}>
        <Text color="red.500">Error loading request details</Text>
      </Box>
    );
  }

  return (
    <Box>
      <Flex mb={6} align="center">
        <IconButton
          as={Link}
          to="/"
          icon={<ArrowBackIcon />}
          aria-label="Back to list"
          variant="ghost"
          mr={4}
        />
        <Box>
          <Heading size="lg" mb={2}>
            Execution Request Details
          </Heading>
          <Text color="gray.600">ID: {request.id}</Text>
        </Box>
      </Flex>

      <SimpleGrid columns={{ base: 1, md: 2 }} spacing={6}>
        <GridItem>
          <Stack spacing={4}>
            <Box p={4} bg="white" rounded="md" shadow="sm">
              <Text fontWeight="bold" mb={2}>
                Status
              </Text>
              {getStatusBadge(request.status)}
            </Box>
            <Box p={4} bg="white" rounded="md" shadow="sm">
              <Text fontWeight="bold" mb={2}>
                Chain Status
              </Text>
              {getChainStatusBadge(request.chainStatus)}
            </Box>
            <Box p={4} bg="white" rounded="md" shadow="sm">
              <Text fontWeight="bold" mb={2}>
                Image ID
              </Text>
              <Code p={2} borderRadius="md" bg="gray.50">
                {request.imageId}
              </Code>
            </Box>
            <Box p={4} bg="white" rounded="md" shadow="sm">
              <Text fontWeight="bold" mb={2}>
                Execution Address
              </Text>
              <Button
                size="sm"
                onClick={onCopy}
                colorScheme={hasCopied ? 'green' : 'gray'}
                mb={2}
              >
                {hasCopied ? 'Copied!' : 'Copy Address'}
              </Button>
              <Code p={2} display="block" borderRadius="md" bg="gray.50">
                {request.executionAddress}
              </Code>
            </Box>
          </Stack>
        </GridItem>

        <GridItem>
          <Stack spacing={4}>
            <Box p={4} bg="white" rounded="md" shadow="sm">
              <Text fontWeight="bold" mb={2}>
                Created At
              </Text>
              <Text>{new Date(request.createdAt).toLocaleString()}</Text>
            </Box>
            <Box p={4} bg="white" rounded="md" shadow="sm">
              <Text fontWeight="bold" mb={2}>
                Updated At
              </Text>
              <Text>{new Date(request.updatedAt).toLocaleString()}</Text>
            </Box>
            {request.proverId && (
              <Box p={4} bg="white" rounded="md" shadow="sm">
                <Text fontWeight="bold" mb={2}>
                  Prover ID
                </Text>
                <Code p={2} borderRadius="md" bg="gray.50">
                  {request.proverId}
                </Code>
              </Box>
            )}
            {request.result && (
              <Box p={4} bg="white" rounded="md" shadow="sm">
                <Text fontWeight="bold" mb={2}>
                  Result
                </Text>
                <Code p={2} display="block" whiteSpace="pre-wrap" borderRadius="md" bg="gray.50">
                  {request.result}
                </Code>
              </Box>
            )}
            {request.error && (
              <Box p={4} bg="white" rounded="md" shadow="sm">
                <Text fontWeight="bold" mb={2}>
                  Error
                </Text>
                <Text color="red.500">{request.error}</Text>
              </Box>
            )}
          </Stack>
        </GridItem>
      </SimpleGrid>

      <Box mt={6} p={4} bg="white" rounded="md" shadow="sm">
        <Text fontWeight="bold" mb={2}>
          Request Data
        </Text>
        <Code
          p={4}
          display="block"
          whiteSpace="pre-wrap"
          borderRadius="md"
          bg="gray.50"
          overflowX="auto"
        >
          {JSON.stringify(JSON.parse(request.requestData), null, 2)}
        </Code>
      </Box>
    </Box>
  );
};

export default RequestDetails; 
